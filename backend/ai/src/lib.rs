// lib.rs

// GA ⇄ Interpreter driver
//
// This file wires to Genetic Algorithm to the external code interpreter, the process is as follows
// For each code generation and each chromosome
// 1) Decode the chromosome bits into an interpreter payload string
// 2) Call the interpreter (runs code, writes to DB, returns per-task outputs)
/// 3) Map outputs to `(ltl_milli, fail_milli)` via `derive_props`
// 4) Compute fitness using Components
// 5) Evolve teh population with the fitness scores
// Notes:
// - The interpreter is called once per chromosome per generation.
// - The Evaluator only checks SOME properties (Safety, Proper
//  Termination, Segfault, Exceptions, Execution Time, Illegal Output). The
//   two “expected output” properties are evaluated elsewhere. (Presumably), not exactly sure how this should be handled
// - `unused_fetch` exists solely for compatibility with the current driver
// signature. We don’t fetch from DB separately because the interpreter
// already returns outputs. However this can be change in the future if we want to run the interpreter in advance

pub mod algorithms {
    pub mod genetic_algorithm;
    pub mod rng;
    pub mod code_coverage;
}

pub mod utils {
    pub mod evaluator;
    pub mod output;
}

use crate::algorithms::genetic_algorithm::{Chromosome, GeneticAlgorithm};
use crate::utils::evaluator::{Evaluator, TaskSpec};
use crate::utils::output::Output;
use code_runner::run_interpreter;
use db::models::assignment_submission::Entity as AssignmentSubmission;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use std::collections::HashMap;
use util::execution_config::ExecutionConfig;

use crate::algorithms::rng::{RandomGenomeGenerator as RngGen, GeneConfig as RngGeneConfig};
use crate::algorithms::code_coverage::{coverage_percent_for_attempt, coverage_fitness};


// -----------------------------------------------------------------------------
// Public entrypoint: build GA + Evaluator + Components, then run the loop
// -----------------------------------------------------------------------------

/// Runs a genetic algorithm for a given submission using an ExecutionConfig
///
/// # Parameters
/// - `db`: shared database connection
/// - `submission_id`: which submission we are optimizing for
/// - `config`: ExecutionConfig containing GA parameters, omegas, language, runtime limits, etc.
///
/// # Behavior
/// - Instantiates the GA population from `config.ga_config`
/// - Instantiates `Components` using omegas from `config`
/// - Builds a `TaskSpec` from `config` for per-task property evaluation
/// - Instantiates an `Evaluator` to check properties like Safety, ProperTermination,
///   SegmentationFault, Exceptions, ExecutionTime, IllegalOutput
/// - Builds a closure `derive_props` that maps interpreter outputs into
///   `(num_ltl_props, num_tasks)` for fitness evaluation
/// - Calls the generic driver `run_ga_end_to_end` which runs the GA loop
///
/// # Returns
/// - `Ok(())` on success, or a `String` error propagated from the interpreter or GA driver
pub async fn run_ga_job(
    db: &DatabaseConnection,
    submission_id: i64,
    config: ExecutionConfig,
    module_id: i64,
    assignment_id: i64,
) -> Result<(), String> {
    // Build GA from ExecutionConfig
    let mut ga = GeneticAlgorithm::from_execution_config(&config.clone());
    let bits_per_gene = ga.bits_per_gene();

    // Fitness Components from omegas
    let (omega1, omega2, omega3) = (
        config.gatlam.omega1,
        config.gatlam.omega2,
        config.gatlam.omega3,
    );
    let mut comps = Components::new(omega1, omega2, omega3, bits_per_gene);

    // Evaluator for property checks
    let evaluator = Evaluator::new();

    // TaskSpec(s) derived from ExecutionConfig
    let base_spec = TaskSpec::from_execution_config(&config);

    let delimiter = config.marking.deliminator.clone();

    let mut derive_props = {
        let evaluator = evaluator;
        let base_spec = base_spec.clone();
        let delim = delimiter.clone();
        move |outs: &[(i64, String)], memo: &[(i64, String)]| -> (usize, usize) {
            let specs = vec![base_spec.clone(); outs.len()];
            evaluator.derive_props(&specs, outs, memo, &delim)
        }
    };

    // Unused fetch closure for signature compatibility
    let mut unused_fetch = |_db: &DatabaseConnection,
                            _sid: i64|
     -> Result<Vec<(i64, String)>, String> { Err("unused".into()) };

    // Run the GA ↔ interpreter loop
    run_ga_end_to_end(
        db,
        submission_id,
        &mut ga,
        &mut comps,
        &mut derive_props,
        &mut unused_fetch,
        module_id,
        assignment_id,
    )
    .await
}

pub async fn run_rng_job(
    db: &DatabaseConnection,
    submission_id: i64,
    config: &ExecutionConfig,
    module_id: i64,
    assignment_id: i64,
    iterations: usize,
    seed: u64,
) -> Result<(), String> {
    let rng_cfgs = exec_to_rng_configs(config);
    let mut generation = RngGen::new(seed);

    // Resolve submission → (user_id, attempt_number)
    let submission = AssignmentSubmission::find_by_id(submission_id)
        .one(db).await
        .map_err(|e| format!("Failed to fetch submission: {}", e))?
        .ok_or_else(|| format!("Submission {} not found", submission_id))?;
    let user_id = submission.user_id;
    let attempt_number = submission.attempt;

    for i in 0..iterations {
        let payload = generation.generate_string(&rng_cfgs);
        eprintln!("[RNG] iter={i} payload={payload}");
        run_interpreter(db, submission_id, &payload).await?;

        // Optional: verify tasks landed (no coverage read here)
        let outs = Output::get_submission_output_no_coverage(
            db, module_id, assignment_id, user_id, attempt_number
        ).await.map_err(|e| e.to_string())?;
        eprintln!("[RNG] iter={i} tasks_returned={}", outs.len());
    }
    Ok(())
}

pub async fn run_coverage_ga_job(
    db: &DatabaseConnection,
    submission_id: i64,
    config: &ExecutionConfig,
    module_id: i64,
    assignment_id: i64,
) -> Result<(), String> {
    let mut ga = GeneticAlgorithm::from_execution_config(&config.clone());
    let bits_per_gene = ga.bits_per_gene();

    let submission = AssignmentSubmission::find_by_id(submission_id)
        .one(db).await
        .map_err(|e| format!("Failed to fetch submission: {}", e))?
        .ok_or_else(|| format!("Submission {} not found", submission_id))?;
    let user_id = submission.user_id;
    let attempt_number = submission.attempt;

    let gens = ga.config().number_of_generations;

    for generation in 0..gens {
        let mut fitness_scores = Vec::with_capacity(ga.population().len());

        for chrom in ga.population().iter() {
            let decoded = decode_genes(chrom.genes(), bits_per_gene);
            let payload = decoded.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",");
            run_interpreter(db, submission_id, &payload).await?;
            let percent = coverage_percent_for_attempt(
                db, module_id, assignment_id, user_id, attempt_number
            ).await?;
            let score = coverage_fitness(percent);

            eprintln!("[CoverageGA] gen={generation} coverage={percent:.2}% score={score:.3}");
            fitness_scores.push(score);
        }

        ga.step_with_fitness(&fitness_scores);
    }

    Ok(())
}

// -----------------------------------------------------------------------------
// Core driver: decode -> interpreter -> derive -> evaluate -> evolve
// -----------------------------------------------------------------------------

/// Generic driver used by `run_ga_job`.
///
/// For each generation:
///   For each chromosome:
///     1) Decode its bits → interpreter payload (comma-separated ints)
///     2) Call the interpreter (async): writes to DB and returns per-task outputs
///     3) Map outputs to `(num_ltl_props, num_tasks)` via `derive_props`
///     4) Compute fitness with `Components` using those counts
///   Then evolve one generation with the collected fitness scores.
///
/// The function is generic over:
/// - `derive_props`: caller-defined mapping from interpreter outputs to counts
///
/// # Errors
/// - Any error from the interpreter or the caller-supplied closures is propagated.
pub async fn run_ga_end_to_end<D, F>(
    db: &DatabaseConnection,
    submission_id: i64,
    ga: &mut GeneticAlgorithm,
    comps: &mut Components,
    mut derive_props: D,
    mut fetch_outputs: F, // kept for compatibility; unused
    module_id: i64,
    assignment_id: i64,
) -> Result<(), String>
where
    // Given raw outputs for this chromosome, return counts the Components expect
    D: FnMut(&[(i64, String)], &[(i64, String)]) -> (usize, usize),
    // Same as mentioned earlier, not used
    F: FnMut(&DatabaseConnection, i64) -> Result<Vec<(i64, String)>, String>,
{
    let _ = &mut fetch_outputs; // suppress unused

    let gens = ga.config().number_of_generations;
    let bits_per_gene = ga.bits_per_gene();

    // Outer loop: generations
    for generation in 0..gens {
        let mut fitness_scores = Vec::with_capacity(ga.population().len());

        // Inner loop: chromosomes in the current population
        for chrom in ga.population().iter() {
            // payload for interpreter, decoded bits into integers into strings
            let decoded = decode_genes(chrom.genes(), bits_per_gene);
            let generated_string = decoded
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");

            // Load submission to get user_id and attempt_number
            let submission = AssignmentSubmission::find_by_id(submission_id)
                .one(db)
                .await
                .map_err(|e| format!("Failed to fetch submission: {}", e))?
                .ok_or_else(|| format!("Submission {} not found", submission_id))?;

            let user_id = submission.user_id;
            let attempt_number = submission.attempt;

            // Run interpreter: executes code for this chromosome, writes artifacts
            //    to DB, and returns per-task outputs for *this* submission.
            //    The interpreter is the source of truth for stdout/stderr/exit codes.
            run_interpreter(db, submission_id, &generated_string).await?;

            let task_outputs: Vec<(i64, String)> = Output::get_submission_output_no_coverage(
                db,
                module_id,
                assignment_id,
                user_id,
                attempt_number,
            )
            .await
            .map_err(|e| e.to_string())?;

            //TODO - LUKE THIS IS CODE COVERAGE

            // let task_code_coverage: Vec<(i64, String)> =
            //     Output::get_submission_output_code_coverage(
            //         db,
            //         module_id,
            //         assignment_id,
            //         user_id,
            //         attempt_number,
            //     )
            //     .await
            //     .map_err(|e| e.to_string())?;

            // let task_code_coverage: Vec<(i64, String)> =
            //     Output::get_submission_output_code_coverage(
            //         db,
            //         module_id,
            //         assignment_id,
            //         user_id,
            //         attempt_number,
            //     )
            //     .await
            //     .map_err(|e| e.to_string())?;

            // let total_coverage_percent =
            //     if let Some((_task_id, coverage_json)) = task_code_coverage.first() {
            //         let parsed: Value = serde_json::from_str(coverage_json)
            //             .map_err(|e| format!("Failed to parse coverage JSON: {}", e))?;
            //         parsed
            //             .get("summary")
            //             .and_then(|s| s.get("coverage_percent"))
            //             .and_then(|v| v.as_f64())
            //             .unwrap_or(0.0)
            //     } else {
            //         0.0
            //     };

            let memo_task_outputs: Vec<(i64, String)> =
                Output::get_memo_output(module_id, assignment_id).map_err(|e| e.to_string())?;

            // Derive counts the Components need:
            //    - `n_ltl_props`: total number of violated properties across tasks
            //    - `num_tasks`  : number of tasks we evaluated
            //    Components will internally normalize (e.g., divide by counts).
            let (ltl_milli, fail_milli) = derive_props(&task_outputs, &memo_task_outputs);

            // Compute fitness for this chromosome in this generation.
            //    `Components` combines sub-scores via omega weights and returns a scalar.
            let score = comps.evaluate(chrom, generation, ltl_milli, fail_milli);
            fitness_scores.push(score);
        }

        // Evolve the population to the next generation using the scores we computed
        ga.step_with_fitness(&fitness_scores);
    }

    Ok(())
}

fn exec_to_rng_configs(cfg: &ExecutionConfig) -> Vec<RngGeneConfig> {
    cfg.gatlam
        .genes
        .iter()
        .map(|g| RngGeneConfig {
            min_value: g.min_value,
            max_value: g.max_value,
            invalid_values: std::collections::HashSet::new(),
        })
        .collect()
}

// Fitness Components
pub struct Components {
    omega1: f64,
    omega2: f64,
    omega3: f64,
    memory_log: HashMap<usize, HashMap<i32, i32>>,
    total_checked: usize,
    bits_per_gene: usize,
}

impl Components {
    pub fn new(omega1: f64, omega2: f64, omega3: f64, bits_per_gene: usize) -> Self {
        if (omega1 + omega2 + omega3 - 1.0).abs() > 1e-6 {
            panic!(
                "Weights of omegas should sum to 1, not {}",
                omega1 + omega2 + omega3
            );
        }
        Self {
            omega1,
            omega2,
            omega3,
            memory_log: HashMap::new(),
            total_checked: 0,
            bits_per_gene,
        }
    }

    /// `num_ltl_props` and `num_tasks` are derived from interpreter outputs upstream.
    pub fn evaluate(
        &mut self,
        x: &Chromosome,
        generation: usize,
        num_ltl_props: usize,
        num_tasks: usize,
    ) -> f64 {
        let ltl = self.compute_ltl(x, num_ltl_props);
        let fail = self.compute_fail(x, num_tasks);
        let mem = self.compute_mem(x, generation);
        self.total_checked += 1;
        if ltl > 0.0 {
            self.update_memory(x);
        }
        self.omega1 * ltl + self.omega2 * fail + self.omega3 * mem
    }

    fn compute_ltl(&self, _x: &Chromosome, n_milli: usize) -> f64 {
        (n_milli as f64 / 1000.0).clamp(0.0, 1.0)
    }

    fn compute_fail(&self, _x: &Chromosome, fail_milli: usize) -> f64 {
        // 0   => 0.0 failure fraction
        // 1000=> 1.0 failure fraction
        (fail_milli as f64 / 1000.0).clamp(0.0, 1.0)
    }

    fn compute_mem(&self, x: &Chromosome, _gen: usize) -> f64 {
        let genes = decode_genes(&x.genes, self.bits_per_gene);
        let mut sum = 0.0;
        for (i, &val) in genes.iter().enumerate() {
            let count = self
                .memory_log
                .get(&i)
                .and_then(|m| m.get(&val))
                .cloned()
                .unwrap_or(0);
            sum += count as f64;
        }
        if genes.is_empty() {
            0.0
        } else {
            sum / (genes.len() as f64 * (self.total_checked.max(1) as f64))
        }
    }

    fn update_memory(&mut self, x: &Chromosome) {
        let genes = decode_genes(&x.genes, self.bits_per_gene);
        for (i, val) in genes.into_iter().enumerate() {
            let entry = self.memory_log.entry(i).or_insert_with(HashMap::new);
            *entry.entry(val).or_insert(0) += 1;
        }
    }
}

// Decoding utilities
/// Decodes a whole chromosome bitstring into i32 values
/// using fixed `bits_per_gene` (sign + magnitude).
pub fn decode_genes(bits: &[bool], bits_per_gene: usize) -> Vec<i32> {
    let mut values = Vec::new();
    let mut index = 0;

    while index + bits_per_gene <= bits.len() {
        let slice = &bits[index..index + bits_per_gene];
        values.push(decode_gene(slice));
        index += bits_per_gene;
    }

    if index != bits.len() {
        panic!(
            "decoding error! (expected multiple of {}, got {})",
            bits_per_gene,
            bits.len()
        );
    }
    values
}

/// First bit is sign; rest are magnitude bits. Returns 0 if too short.
fn decode_gene(bits: &[bool]) -> i32 {
    if bits.len() < 2 {
        return 0;
    }
    let is_negative = bits[0];
    let mut val = 0i32;
    for &b in &bits[1..] {
        val = (val << 1) | (b as i32);
    }
    let max = (1 << (bits.len() - 1)) - 1;
    let decoded = if is_negative { -val } else { val };
    decoded.clamp(-max, max)
}

#[cfg(test)]
mod component_unit_tests {
    use super::*;
    use crate::algorithms::genetic_algorithm::Chromosome;

    // local helper: encode a single i32 into sign+magnitude (MSB-first) bits
    fn encode_val(v: i32, bits_per_gene: usize) -> Vec<bool> {
        assert!(bits_per_gene >= 2, "need >= 2 bits (sign + >=1 magnitude)");
        let sign = v < 0;
        let mag_bits = bits_per_gene - 1;
        let mut out = Vec::with_capacity(bits_per_gene);
        out.push(sign);
        let m = (v.abs() as u32) & ((1u32 << mag_bits) - 1);
        for i in (0..mag_bits).rev() {
            out.push(((m >> i) & 1) == 1);
        }
        out
    }

    fn chrom_from_vals(vals: &[i32], bits_per_gene: usize) -> Chromosome {
        let mut bits = Vec::new();
        for &v in vals {
            bits.extend(encode_val(v, bits_per_gene));
        }
        Chromosome::new(bits)
    }

    #[test]
    fn compute_ltl_maps_milli_fraction() {
        let mut comps = Components::new(0.5, 0.3, 0.2, 4);
        // ltl=0.0
        let s0 = comps.evaluate(&chrom_from_vals(&[1, 2], 4), 0, 0, 0);
        // ltl=0.5
        let s1 = comps.evaluate(&chrom_from_vals(&[1, 2], 4), 0, 500, 0);
        // ltl=1.0
        let s2 = comps.evaluate(&chrom_from_vals(&[1, 2], 4), 0, 1000, 0);
        assert!(s0 < s1 && s1 < s2, "scores should increase with ltl_milli");
    }

    #[test]
    fn compute_fail_maps_milli_fraction() {
        let mut comps = Components::new(0.0, 1.0, 0.0, 4); // only 'fail' weight
        // 0.0 -> 0
        let s0 = comps.evaluate(&chrom_from_vals(&[1], 4), 0, 0, 0);
        // 0.3 -> 300/1000
        let s1 = comps.evaluate(&chrom_from_vals(&[1], 4), 0, 0, 300);
        // 1.0 -> 1000/1000
        let s2 = comps.evaluate(&chrom_from_vals(&[1], 4), 0, 0, 1000);
        assert!((s0 - 0.0).abs() < 1e-9);
        assert!((s1 - 0.3).abs() < 1e-9);
        assert!((s2 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn decode_genes_decodes_sign_and_magnitude() {
        // bits_per_gene = 4 (1 sign + 3 magnitude)
        // vals: [ 3, -2, 0 ]  => ensure sign handling + clamping path is stable
        let bpg = 4;
        let c = chrom_from_vals(&[3, -2, 0], bpg);
        let decoded = super::decode_genes(c.genes(), bpg);
        assert_eq!(decoded, vec![3, -2, 0]);
    }

    #[test]
    fn memory_component_increases_when_ltl_positive() {
        // mem contributes only when ltl > 0.0 triggers update_memory()
        let bpg = 4;
        let mut comps = Components::new(0.0, 0.0, 1.0, bpg); // only memory weight
        let ch = chrom_from_vals(&[3, 3, 3], bpg);

        let s0 = comps.evaluate(&ch, 0, 0, 0);
        // Now call with ltl>0 => update_memory occurs
        let _ = comps.evaluate(&ch, 0, 1000, 0);
        // Another evaluation should see higher memory score than first one
        let s2 = comps.evaluate(&ch, 1, 0, 0);

        assert!(
            s2 > s0,
            "memory score should increase after a positive LTL triggers update_memory"
        );
    }

    #[test]
    fn components_weights_must_sum_to_one() {
        let _ok = Components::new(0.3, 0.3, 0.4, 4);
    }

    #[test]
    #[should_panic]
    fn components_weights_invalid_sum_panics() {
        let _ = Components::new(0.5, 0.5, 0.5, 4);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use sea_orm::{Database, DatabaseConnection};
//     use std::collections::HashSet;
//     #[tokio::test(flavor = "multi_thread")]
//     async fn smoke_run_ga_on_submission_9998() {
//         dotenv::dotenv().ok();

//         // Check for opt-in flag
//         if std::env::var("GA_ITEST").is_err() {
//             eprintln!("GA_ITEST not set; skipping smoke test.");
//             return;
//         }

//         // Connect to DB
//         let db = connect().await;

//        // Minimal GA config (tiny population / generations for a fast run)
//         let genes = vec![
//             GeneConfig {
//                 min_value: -5,
//                 max_value: 5,
//                 invalid_values: HashSet::new(),
//             },
//             GeneConfig {
//                 min_value: -4,
//                 max_value: 9,
//                 invalid_values: HashSet::new(),
//             },
//         ];

//         let ga_config = GAConfig {
//             population_size: 4,
//             number_of_generations: 1,
//             selection_size: 2,
//             reproduction_probability: 0.9,
//             crossover_probability: 0.8,
//             mutation_probability: 0.05,
//             genes,
//             crossover_type: CrossoverType::Uniform,
//             mutation_type: MutationType::BitFlip,
//         };

//         let (omega1, omega2, omega3) = (0.4, 0.4, 0.2);

//         let submission_id: i64 = 191;

//         let res = run_ga_job(&db, submission_id, ExecutionConfig::default_config()).await;

//         match res {
//             Ok(()) => eprintln!("smoke_run_ga_on_submission_9998: OK"),
//             Err(e) => eprintln!("smoke_run_ga_on_submission_9998: ERR: {e}"),
//         }
//     }
// }

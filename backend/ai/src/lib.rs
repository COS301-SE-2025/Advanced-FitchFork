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
}

pub mod utils {
    pub mod evaluator;
}


use crate::algorithms::genetic_algorithm::{GeneticAlgorithm, Chromosome, GAConfig, GeneConfig, CrossoverType, MutationType};
use crate::utils::evaluator::{Evaluator, TaskSpec, Language};
use code_runner::run_interpreter;
use std::collections::HashMap;
use db::{connect};
use sea_orm::{Database, DatabaseConnection};



// -----------------------------------------------------------------------------
// Public entrypoint: build GA + Evaluator + Components, then run the loop
// -----------------------------------------------------------------------------

/// Runs a genetic algorithm for a given submission
/// # Parameters
/// - `db`: shared DB connection (SeaORM)
/// - `submission_id`: which submission we’re optimizing for
/// - `ga_config`: fully-configured GA (population size, gens, crossover, etc.)
/// - `omega1..3`: weights for the Components fitness aggregation (must sum to ~1)
///
/// # Behavior
/// - Instantiates the GA population and fitness `Components`
/// - Instantiates an `Evaluator` and a base `TaskSpec` (rules for per-task checks)
/// - Builds a closure `derive_props` that translates interpreter outputs into
///   `(num_ltl_props, num_tasks)` *per chromosome*, which `Components` expects
/// - Calls the generic driver `run_ga_end_to_end`
///
/// # Returns
/// - `Ok(())` on success, or a `String` error propagated from the interpreter/driver
pub async fn run_ga_job(
    db: &DatabaseConnection,
    submission_id: i64,
    ga_config: GAConfig,
    omega1: f64,
    omega2: f64,
    omega3: f64,
) -> Result<(), String> {

    // Build core GA (population + config)
    let mut ga = GeneticAlgorithm::new(ga_config);

    // Number of bits to decode per gene, needed for payload decoding
    let bits_per_gene = ga.bits_per_gene();

    // Fitness components 
    let mut comps = Components::new(omega1, omega2, omega3, bits_per_gene);

    // Evaluator translates raw interpreter outputs into property-violation counts
    // (Safety, ProperTermination, Segfault, Exceptions, ExecutionTime, IllegalOutput)
    let evaluator = Evaluator::new();
    

    // Base rules for evaluation each task's outputs
    // replace this single spec with a per-task vector pulled from DB/config.
    let base_spec = TaskSpec {
        language: Language::Cpp,
        valid_return_codes: Some(vec![0]),
        max_runtime_ms: None,          
        forbidden_outputs: vec![],   
    }; // TODO: make this configurable later

    // This closure is invoked inside the GA loop for every chromosome’s run:
    // it converts raw outputs into `(num_ltl_props, num_tasks)`.
    let mut derive_props = move |outs: &[(i64, String)]| -> (usize, usize) {
        // If all tasks share the same rules, replicate the same spec per task.
        // If tasks differ, replace with a task-specific `Vec<TaskSpec>`.
        let specs = vec![base_spec.clone(); outs.len()];
        evaluator.derive_props(&specs, outs)
    };
    // Signature compatibility shim (unused in current flow).
    // `run_ga_end_to_end` still requires a `fetch_outputs` parameter, but we
    // call the interpreter directly and already have the outputs in memory.
    // We can change this later if we want to fetch outputs from DB separately.
    // Such in the case that the interpreter runs in advance
    let mut unused_fetch = |_db: &DatabaseConnection, _sid: i64| -> Result<Vec<(i64, String)>, String> {
        Err("unused".into())
    };

    // Drive the GA ↔ interpreter loop
    run_ga_end_to_end(
        db,
        submission_id,
        &mut ga,
        &mut comps,
        &mut derive_props,
        &mut unused_fetch,
    ).await
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
) -> Result<(), String>
where
    // Given raw outputs for this chromosome, return counts the Components expect
    D: FnMut(&[(i64, String)]) -> (usize, usize),
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

            // Run interpreter: executes code for this chromosome, writes artifacts
            //    to DB, and returns per-task outputs for *this* submission.
            //    The interpreter is the source of truth for stdout/stderr/exit codes.
            let task_outputs: Vec<(i64, String)> =
                run_interpreter(db, submission_id, &generated_string).await?;

            // Derive counts the Components need:
            //    - `n_ltl_props`: total number of violated properties across tasks
            //    - `num_tasks`  : number of tasks we evaluated
            //    Components will internally normalize (e.g., divide by counts).
            let (ltl_milli, fail_milli) = derive_props(&task_outputs);

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
            panic!("Weights of omegas should sum to 1, not {}", omega1 + omega2 + omega3);
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
    pub fn evaluate(&mut self, x: &Chromosome, generation: usize, num_ltl_props: usize, num_tasks: usize) -> f64 {
        let ltl  = self.compute_ltl(x, num_ltl_props);
        let fail = self.compute_fail(x, num_tasks);
        let mem  = self.compute_mem(x, generation);
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
        if genes.is_empty() { 0.0 } else {
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

    fn simulate_task_failure(&self, _x: &Chromosome, _i: usize) -> bool {
        rand::random::<f64>() < 0.3
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
    if bits.len() < 2 { return 0; }
    let is_negative = bits[0];
    let mut val = 0i32;
    for &b in &bits[1..] { val = (val << 1) | (b as i32); }
    let max = (1 << (bits.len() - 1)) - 1;
    let decoded = if is_negative { -val } else { val };
    decoded.clamp(-max, max)
}



#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DatabaseConnection};
    use std::collections::HashSet;
    #[tokio::test(flavor = "multi_thread")]
    async fn smoke_run_ga_on_submission_9998() {
 
        dotenv::dotenv().ok();

        if std::env::var("GA_ITEST").is_err() {
            return;
        }

        let db = connect().await;
        let genes = vec![
            GeneConfig { min_value: -5, max_value: 5, invalid_values: HashSet::new() },
            GeneConfig { min_value: -4, max_value: 9, invalid_values: HashSet::new() },
        ];

        let ga_config = GAConfig {
            population_size: 4,
            number_of_generations: 3,
            selection_size: 2,
            reproduction_probability: 0.9,
            crossover_probability: 0.8,
            mutation_probability: 0.05,
            genes,
            crossover_type: CrossoverType::Uniform,
            mutation_type: MutationType::BitFlip,
        };

        let (omega1, omega2, omega3) = (0.4, 0.4, 0.2);

        let submission_id: i64 = 602;

        let res = run_ga_job(&db, submission_id, ga_config, omega1, omega2, omega3).await;
    }
}
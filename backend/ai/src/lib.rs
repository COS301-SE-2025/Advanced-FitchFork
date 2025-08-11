// lib.rs


pub mod algorithms {
    pub mod genetic_algorithm;
}

pub mod utils {
    pub mod evaluator;
}

use crate::algorithms::genetic_algorithm::{GeneticAlgorithm, Chromosome, GAConfig, GeneConfig, CrossoverType, MutationType};
use crate::utils::evaluator::{Evaluator, TaskSpec, Language};

// --- External deps ---
use sea_orm::DatabaseConnection;
use code_runner::run_interpreter;
use std::collections::HashMap;

pub async fn run_ga_job(
    db: &DatabaseConnection,
    submission_id: i64,
    ga_config: GAConfig,
    omega1: f64,
    omega2: f64,
    omega3: f64,
) -> Result<(), String> {
    let mut ga = GeneticAlgorithm::new(ga_config);
    let bits_per_gene = ga.bits_per_gene();
    let mut comps = Components::new(omega1, omega2, omega3, bits_per_gene);

    let mut evaluator = Evaluator::new();
    
    let base_spec = TaskSpec {
        language: Language::Cpp,
        valid_return_codes: Some(vec![0]),
        max_runtime_ms: None,          
        forbidden_outputs: vec![],   
    }; // TODO: make this configurable later

    let mut derive_props = move |outs: &[(i64, String)]| -> (usize, usize) {
        // if all tasks share the same spec:
        let specs = vec![base_spec.clone(); outs.len()];
        evaluator.derive_props(&specs, outs)
    };

    // Signature compatibility
    let mut unused_fetch = |_db: &DatabaseConnection, _sid: i64| -> Result<Vec<(i64, String)>, String> {
        Err("unused".into())
    };

    run_ga_end_to_end(
        db,
        submission_id,
        &mut ga,
        &mut comps,
        &mut derive_props,
        &mut unused_fetch,
    ).await
}

/// Core driver: decode -> interpreter -> derive → evaluate -> evolve.
pub async fn run_ga_end_to_end<D, F>(
    db: &DatabaseConnection,
    submission_id: i64,
    ga: &mut GeneticAlgorithm,
    comps: &mut Components,
    mut derive_props: D,
    mut fetch_outputs: F, // kept for compatibility; unused
) -> Result<(), String>
where
    D: FnMut(&[(i64, String)]) -> (usize, usize),
    F: FnMut(&DatabaseConnection, i64) -> Result<Vec<(i64, String)>, String>,
{
    let _ = &mut fetch_outputs; // suppress unused

    let gens = ga.config().number_of_generations;
    let bits_per_gene = ga.bits_per_gene();

    for generation in 0..gens {
        let mut fitness_scores = Vec::with_capacity(ga.population().len());

        for chrom in ga.population().iter() {
            // payload for interpreter
            let decoded = decode_genes(chrom.genes(), bits_per_gene);
            let generated_string = decoded
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");

            // external interpreter 
            let task_outputs: Vec<(i64, String)> =
                run_interpreter(db, submission_id, &generated_string).await?;

            // map outputs -> fitness params
            let (n_ltl_props, num_tasks) = derive_props(&task_outputs);

            // compute fitness
            let score = comps.evaluate(chrom, generation, n_ltl_props, num_tasks);
            fitness_scores.push(score);
        }

        // evolve
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

    fn compute_ltl(&self, _x: &Chromosome, n: usize) -> f64 {
        let mut violations = 0;
        for i in 0..n {
            if self.simulate_tl_property_violation(_x, i) { //todo: return actual ltl violation from loop
                violations += 1;
            }
        }
        violations as f64 / (n.max(1) as f64)
    }

    fn compute_fail(&self, _x: &Chromosome, m: usize) -> f64 {
        let mut failed = 0;
        for i in 0..m {
            if self.simulate_task_failure(_x, i) { //todo: return actual fail from loop
                failed += 1;
            }
        }
        failed as f64 / (m.max(1) as f64)
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

    fn simulate_tl_property_violation(&self, _x: &Chromosome, _i: usize) -> bool {
        rand::random::<f64>() < 0.2
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


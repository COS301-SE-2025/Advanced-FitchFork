//these are the folders that exist with mod.rs files in them
mod algorithms;
mod utils;
use crate::algorithms::genetic_algorithm::{GAConfig, GeneticAlgorithm, Chromosome};
use std::collections::HashSet;


pub fn construct_ga(config: GAConfig, omegas: (f64, f64, f64)) -> GeneticAlgorithm {
    let (w1, w2, w3) = omegas;
    GeneticAlgorithm::new(config, w1, w2, w3)
}

pub fn run_ga_loop<F>(mut ga: GeneticAlgorithm, mut fetch_run_params: F) -> GeneticAlgorithm where F: FnMut() -> (usize, usize), {
    let (n_ltl_props, n_tasks) = fetch_run_params();



    run_interpreter(&as_string);

    ga.run(n_ltl_props, n_tasks);
    ga

}

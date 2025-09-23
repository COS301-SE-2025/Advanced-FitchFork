use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;
use util::execution_config::ExecutionConfig;

#[derive(Debug, Clone)]
pub struct GeneConfig {
    pub min_value: i32,
    pub max_value: i32,
    pub invalid_values: HashSet<i32>,
}

impl GeneConfig {
    pub fn bits(&self) -> usize {
        let max_abs = self.min_value.abs().max(self.max_value.abs()) as f64;
        max_abs.log2().ceil() as usize + 1
    }
}
pub struct RandomGenomeGenerator {
    rng: StdRng,
}

impl RandomGenomeGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn generate_string(&mut self, configs: &[GeneConfig]) -> String {
        let mut values = Vec::with_capacity(configs.len());
        for cfg in configs {
            let v = self.random_gene_value(cfg);
            values.push(v);
        }
        values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    fn random_gene_value(&mut self, cfg: &GeneConfig) -> i32 {
        loop {
            let v = self.rng.gen_range(cfg.min_value..=cfg.max_value);
            if !cfg.invalid_values.contains(&v) {
                return v;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(min: i32, max: i32, invalid: &[i32]) -> GeneConfig {
        let set: HashSet<i32> = invalid.iter().cloned().collect();
        GeneConfig {
            min_value: min,
            max_value: max,
            invalid_values: set,
        }
    }

    #[test]
    fn test_single_gene_full_range() {
        let cfg = make_config(0, 3, &[]);
        let mut generation = RandomGenomeGenerator::new(42);
        let s = generation.generate_string(&[cfg]);
        let vals: Vec<i32> = s.split(',').map(|x| x.parse().unwrap()).collect();
        assert_eq!(vals.len(), 1);
        assert!((0..=3).contains(&vals[0]));
    }

    #[test]
    fn test_single_gene_excluding_values() {
        let cfg = make_config(1, 5, &[2, 3]);
        let mut generation = RandomGenomeGenerator::new(100);
        for _ in 0..10 {
            let val: i32 = generation.generate_string(&[cfg.clone()]).parse().unwrap();
            assert!(val >= 1 && val <= 5);
            assert!(val != 2 && val != 3);
        }
    }

    #[test]
    fn test_multiple_genes_string_format() {
        let cfg1 = make_config(-1, 1, &[]);
        let cfg2 = make_config(10, 12, &[11]);
        let mut generation = RandomGenomeGenerator::new(7);
        let s = generation.generate_string(&[cfg1, cfg2]);
        let parts: Vec<&str> = s.split(',').collect();
        assert_eq!(parts.len(), 2);
        let v1: i32 = parts[0].parse().unwrap();
        let v2: i32 = parts[1].parse().unwrap();
        assert!(v1 >= -1 && v1 <= 1);
        assert!(v2 >= 10 && v2 <= 12 && v2 != 11);
    }

    #[test]
    fn test_bits_calculation_positive() {
        let cfg = make_config(0, 7, &[]);
        // max_abs = 7, log2(7)=2.8, ceil=3 + sign bit = 4
        assert_eq!(cfg.bits(), 4);
    }

    #[test]
    fn test_bits_calculation_negative() {
        let cfg = make_config(-8, -1, &[]);
        // max_abs = 8, log2(8)=3, ceil=3 + sign bit = 4
        assert_eq!(cfg.bits(), 4);
    }

    #[test]
    fn test_empty_configs() {
        let mut generation = RandomGenomeGenerator::new(1);
        let s = generation.generate_string(&[]);
        assert!(s.is_empty());
    }

    #[test]
    fn test_single_valid_only() {
        let cfg = make_config(0, 2, &[0, 1]);
        let mut generation = RandomGenomeGenerator::new(7);
        for _ in 0..5 {
            let val: i32 = generation.generate_string(&[cfg.clone()]).parse().unwrap();
            assert_eq!(val, 2);
        }
    }

    #[test]
    fn test_zero_range() {
        let cfg = make_config(5, 5, &[]);
        let mut generation = RandomGenomeGenerator::new(9);
        for _ in 0..3 {
            let val: i32 = generation.generate_string(&[cfg.clone()]).parse().unwrap();
            assert_eq!(val, 5);
        }
    }

    #[test]
    fn test_negative_range() {
        let cfg = make_config(-3, -1, &[]);
        let mut generation = RandomGenomeGenerator::new(123);
        for _ in 0..10 {
            let val: i32 = generation.generate_string(&[cfg.clone()]).parse().unwrap();
            assert!((-3..=-1).contains(&val));
        }
    }

    #[test]
    fn test_deterministic_seed() {
        let cfgs = vec![make_config(0, 3, &[]), make_config(5, 7, &[6])];
        let mut gen1 = RandomGenomeGenerator::new(555);
        let mut gen2 = RandomGenomeGenerator::new(555);
        assert_eq!(gen1.generate_string(&cfgs), gen2.generate_string(&cfgs));
    }
}

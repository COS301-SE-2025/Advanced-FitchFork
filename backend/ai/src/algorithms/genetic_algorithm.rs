use rand::seq::SliceRandom;
use rand::{Rng, thread_rng};
use std::collections::HashSet;
use util::execution_config::ExecutionConfig;
use util::execution_config::{
    CrossoverType as ExecCrossoverType, MutationType as ExecMutationType,
};

/// Gene-level configuration
#[derive(Clone)]
pub struct GeneConfig {
    pub min_value: i32,               // minimum valid value
    pub max_value: i32,               // maximum valid value
    pub invalid_values: HashSet<i32>, // explicitly disallowed values
}

impl GeneConfig {
    // creates a new gene config
    pub fn new(min_value: i32, max_value: i32, invalid_values: HashSet<i32>) -> Self {
        Self {
            min_value,
            max_value,
            invalid_values,
        }
    }

    // calculates the number of bits needed to represent the gene value
    pub fn bits(&self) -> usize {
        ((self.max_value.abs().max(self.min_value.abs()) as f64).log2()).ceil() as usize + 1
    }
}

/// GA-wide configuration
pub struct GAConfig {
    pub population_size: usize,       // Number of chromosomes per generation
    pub number_of_generations: usize, // Total number of generations to run
    //todo: use this in run
    pub selection_size: usize, // Number of individuals selected during selection
    pub reproduction_probability: f64, // Probability of applying crossover during reproduction
    //todo: use this probability
    pub crossover_probability: f64, // Used in some variants, may control forced crossover
    pub mutation_probability: f64,  // Probability of mutating a child
    pub genes: Vec<GeneConfig>,     // Configuration for each gene in the chromosome
    pub crossover_type: CrossoverType, // Which crossover operator to use (one-point, two-point, uniform)
    pub mutation_type: MutationType,   // Which mutation operator to use (bit-flip, swap, scramble)
}

impl GAConfig {
    // constructor with above fields
    pub fn new(
        population_size: usize,
        number_of_generations: usize,
        selection_size: usize,
        reproduction_probability: f64,
        crossover_probability: f64,
        mutation_probability: f64,
        genes: Vec<GeneConfig>,
        crossover_type: CrossoverType,
        mutation_type: MutationType,
    ) -> Self {
        Self {
            population_size,
            number_of_generations,
            selection_size,
            reproduction_probability,
            crossover_probability,
            mutation_probability,
            genes,
            crossover_type,
            mutation_type,
        }
    }

    // calculates the number of bits needed to represent all genes in the chromosome
    // this is the sum of bits for each gene, it is used to determine the length of the chromosome bit string
    pub fn bits(&self) -> usize {
        let mut min_value = i32::MAX;
        let mut max_value = i32::MIN;

        // find global min and max values across all genes
        for gene_config in &self.genes {
            min_value = min_value.min(gene_config.min_value);
            max_value = max_value.max(gene_config.max_value);
        }

        // calculate bits needed to represent the largest absolute value
        ((max_value.abs().max(min_value.abs()) as f64).log2()).ceil() as usize + 1
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CrossoverType {
    OnePoint,
    TwoPoint,
    Uniform,
}

#[derive(Clone, Copy, Debug)]
pub enum MutationType {
    BitFlip,
    Swap,
    Scramble,
}

/// Bitstring chromosome
#[derive(Clone)]
pub struct Chromosome {
    pub genes: Vec<bool>,
}

impl Chromosome {
    pub fn new(genes: Vec<bool>) -> Self {
        Self { genes }
    }

    pub fn genes(&self) -> &Vec<bool> {
        &self.genes
    }

    pub fn set_genes(&mut self, genes: Vec<bool>) {
        self.genes = genes;
    }

    pub fn genes_mut(&mut self) -> &mut Vec<bool> {
        &mut self.genes
    }
}

pub struct GeneticAlgorithm {
    population: Vec<Chromosome>,
    generation: usize,
    config: GAConfig,
    bits_per_gene: usize,
}

impl GeneticAlgorithm {
    /// Create a new GA instance using parameters from ExecutionConfig
    pub fn from_execution_config(config: &ExecutionConfig) -> Self {
        let gatlam = &config.gatlam;

        let crossover_type = match gatlam.crossover_type {
            ExecCrossoverType::OnePoint => CrossoverType::OnePoint,
            ExecCrossoverType::TwoPoint => CrossoverType::TwoPoint,
            ExecCrossoverType::Uniform => CrossoverType::Uniform,
        };

        let mutation_type = match gatlam.mutation_type {
            ExecMutationType::BitFlip => MutationType::BitFlip,
            ExecMutationType::Swap => MutationType::Swap,
            ExecMutationType::Scramble => MutationType::Scramble,
        };

        // Convert GeneConfig if needed
        let genes = gatlam
            .genes
            .iter()
            .map(|g| {
                let invalids = HashSet::new(); // or copy invalids if available
                GeneConfig::new(g.min_value, g.max_value, invalids)
            })
            .collect();

        let ga_config = GAConfig::new(
            gatlam.population_size,
            gatlam.number_of_generations,
            gatlam.selection_size,
            gatlam.reproduction_probability,
            gatlam.crossover_probability,
            gatlam.mutation_probability,
            genes,
            crossover_type,
            mutation_type,
        );

        Self::new(ga_config)
    }

    pub fn new(config: GAConfig) -> Self {
        let population = Self::initialize_population(&config);
        let bits_per_gene = config.bits();
        GeneticAlgorithm {
            population,
            generation: 0,
            config,
            bits_per_gene,
        }
    }

    /// Evolve one generation using externally computed fitness scores.
    /// `fitness_scores.len()` must equal `self.population.len()`.
    pub fn step_with_fitness(&mut self, fitness_scores: &[f64]) {
        let next_gen = self.build_next_generation(fitness_scores);
        self.population = next_gen;
        self.generation += 1;
    }

    fn build_next_generation(&self, fitness_scores: &[f64]) -> Vec<Chromosome> {
        assert_eq!(
            fitness_scores.len(),
            self.population.len(),
            "fitness/pop size mismatch"
        );
        let total_fitness: f64 = fitness_scores.iter().sum();

        let mut next_gen = Vec::with_capacity(self.population.len());
        let mut rng = thread_rng();

        // generate the next generation
        // using roulette wheel selection and crossover/mutation
        for _ in 0..self.population.len() {
            // with a probability, select two parents and crossover
            // otherwise, clone one parent without crossover
            // this is a form of elitism where some chromosomes are directly passed to the next generation
            // this is done to maintain diversity in the population and to ensure that the best solutions are not lost
            // roulette wheel selection based on fitness scores
            let mut child = if rng.gen_range(0.0..1.0) < self.config.reproduction_probability {
                let p1 = Self::roulette(&self.population, fitness_scores, total_fitness);
                let p2 = Self::roulette(&self.population, fitness_scores, total_fitness);
                Self::crossover(&p1, &p2, self.config.crossover_type)
            } else {
                let p = Self::roulette(&self.population, fitness_scores, total_fitness);
                Chromosome::new(p.genes().clone())
            };

            // mutate the child with a probability
            if rng.r#gen::<f64>() < self.config.mutation_probability {
                Self::mutate(
                    &mut child,
                    self.config.mutation_type,
                    self.config.mutation_probability,
                );
            }
            // add the child to the next generation
            next_gen.push(child);
        }

        next_gen
    }

    fn initialize_population(config: &GAConfig) -> Vec<Chromosome> {
        let mut rng = thread_rng(); // random number generator
        let mut pop = Vec::with_capacity(config.population_size); // allocate space for population

        let num_genes = config.genes.len();
        let bits_per_gene = config.bits();

        // Create each individual
        for _ in 0..config.population_size {
            let mut gene_bits = Vec::with_capacity(bits_per_gene * num_genes);

            // Generate each gene based on its specific GeneConfig
            for gene_config in &config.genes {
                let gene;
                // Retry until we get a valid gene (not in invalid_values)
                loop {
                    let candidate = rng.gen_range(gene_config.min_value..=gene_config.max_value);
                    if !gene_config.invalid_values.contains(&candidate) {
                        gene = candidate;
                        break;
                    }
                }
                gene_bits.extend(encode_gene(gene, bits_per_gene)); // encode and append bits
            }

            pop.push(Chromosome::new(gene_bits)); // Add to population
        }

        pop
    }

    fn roulette(population: &[Chromosome], fitness: &[f64], total: f64) -> Chromosome {
        let mut rng = thread_rng(); // random number generator
        let mut cumulative = 0.0; // fitness cumulative sum
        let pick = rng.r#gen::<f64>() * total; // random pick in the range of total fitness

        // iterate through the population and fitness scores
        // accumulate the fitness scores until we reach the random pick
        for (i, &f) in fitness.iter().enumerate() {
            cumulative += f;
            if cumulative >= pick {
                return population[i].clone(); // return the chromosome corresponding to the selected fitness
            }
        }
        population.last().unwrap().clone() // fallback to the last chromosome if no selection was made
    }

    fn crossover(p1: &Chromosome, p2: &Chromosome, ty: CrossoverType) -> Chromosome {
        let g1 = p1.genes(); // get genes from the first parent
        let g2 = p2.genes(); // get genes from the second parent
        let len = g1.len(); // chromosome length
        let mut rng = thread_rng(); // random number generator 
        let mut child = Vec::with_capacity(len); // new chromosome to be built 

        match ty {
            CrossoverType::OnePoint => {
                let pt = rng.gen_range(0..len); // choose crossover point
                for i in 0..len {
                    // copy from p1 before pt, from p2 after
                    child.push(if i < pt { g1[i] } else { g2[i] });
                }
            }
            CrossoverType::TwoPoint => {
                // choose two crossover points
                let a = rng.gen_range(0..len);
                let b = rng.gen_range(0..len);
                let (start, end) = if a < b { (a, b) } else { (b, a) }; // ebsyre start <= end
                for i in 0..len {
                    // middle segment from p2, rest from p1
                    child.push(if i < start || i > end { g1[i] } else { g2[i] });
                }
            }
            CrossoverType::Uniform => {
                // uniform crossover: each gene is chosen randomly from either parent
                for i in 0..len {
                    child.push(if rng.r#gen() { g1[i] } else { g2[i] });
                }
            }
        }
        Chromosome::new(child) // return the new child chromosome
    }

    fn mutate(c: &mut Chromosome, ty: MutationType, mutation_prob: f64) {
        let mut rng = thread_rng(); // random number generator
        let genes = c.genes_mut(); // get mutable reference to genes

        match ty {
            MutationType::BitFlip => {
                // flip each bit with a given probability
                for bit in genes.iter_mut() {
                    if rng.r#gen::<f64>() < mutation_prob {
                        *bit = !*bit;
                    }
                }
            }
            MutationType::Swap => {
                // swap two random genes in the chromosome
                let len = genes.len();
                if len >= 2 {
                    let i = rng.gen_range(0..len); // pick first index
                    let mut j = rng.gen_range(0..len); // pick second index
                    while j == i {
                        j = rng.gen_range(0..len); // ensure j =/ i
                    }
                    genes.swap(i, j); // swap
                }
            }
            MutationType::Scramble => {
                // scramble a random segment of genes
                let len = genes.len();
                if len >= 2 {
                    let start = rng.gen_range(0..len);
                    let end = rng.gen_range(start..len);
                    genes[start..=end].shuffle(&mut rng); // shuffle the segment
                }
            }
        }
    }

    // getters for external driver
    pub fn population(&self) -> &[Chromosome] {
        &self.population
    }
    pub fn generation(&self) -> usize {
        self.generation
    }
    pub fn config(&self) -> &GAConfig {
        &self.config
    }
    pub fn bits_per_gene(&self) -> usize {
        self.bits_per_gene
    }
}

/// Bit-level encoder (sign + magnitude)
fn encode_gene(value: i32, bits: usize) -> Vec<bool> {
    let mut binary = Vec::with_capacity(bits);
    let is_negative = value < 0;
    let magnitude_bits = bits - 1;
    let abs_val = value.abs() as u32;

    // Truncate to fit within available bits
    let truncated = abs_val & ((1 << magnitude_bits) - 1);

    binary.push(is_negative); // sign bit

    for i in (0..magnitude_bits).rev() {
        binary.push((truncated >> i) & 1 == 1);
    }

    binary
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn hs(v: &[i32]) -> HashSet<i32> {
        v.iter().cloned().collect()
    }

    /// Test-only decode that matches our sign+magnitude encoder.
    fn test_decode_gene(bits: &[bool]) -> i32 {
        if bits.is_empty() {
            return 0;
        }
        let sign = bits[0];
        let mut mag = 0i32;
        for &b in &bits[1..] {
            mag = (mag << 1) | (b as i32);
        }
        if sign { -mag } else { mag }
    }

    // --- GeneConfig & GAConfig ----------------------------------------------

    #[test]
    fn gene_bits_respects_range() {
        // Max abs=9 -> ceil(log2(9))=4, +1 sign = 5 bits
        let g = GeneConfig::new(-9, 9, HashSet::new());
        assert_eq!(g.bits(), 5);
        let g2 = GeneConfig::new(-1, 1, HashSet::new());
        assert_eq!(g2.bits(), 1);
    }

    #[test]
    fn ga_bits_uses_global_max_abs() {
        let genes = vec![
            GeneConfig::new(-3, 3, HashSet::new()), // needs 3 bits (ceil(log2(3))=2 + sign =3)
            GeneConfig::new(-9, 9, HashSet::new()), // needs 5 bits
            GeneConfig::new(-2, 7, HashSet::new()), // needs 4 bits
        ];
        let cfg = GAConfig::new(
            8,
            10,
            4,
            0.9,
            0.8,
            0.05,
            genes,
            CrossoverType::Uniform,
            MutationType::BitFlip,
        );
        // Global max abs is 9 -> 5 bits total per gene
        assert_eq!(cfg.bits(), 5);
    }

    #[test]
    fn encode_gene_sign_magnitude_roundtrip_like() {
        // Using test-only decoder to validate encoder shape.
        let bits = 5;
        for v in [-9, -1, 0, 1, 7, 9] {
            let enc = super::encode_gene(v, bits);
            assert_eq!(enc.len(), bits);

            // Note: encoder truncates magnitude to available bits; our test decode
            // reconstructs within that limited width, which matches expectations.
            let dec = test_decode_gene(&enc);
            // dec may differ if v was out-of-range for the given bit width.
            // But for |v| <= 2^(bits-1)-1 it should match exactly.
            let max = (1 << (bits - 1)) - 1;
            if v.abs() as i32 <= max {
                assert_eq!(dec, v);
            } else {
                assert_eq!(dec.abs() as i32, max);
            }
        }
    }
    #[test]
    fn initialize_population_shapes_are_correct() {
        let genes = vec![
            GeneConfig::new(-3, 3, hs(&[2])), // disallow 2
            GeneConfig::new(-9, 9, HashSet::new()),
        ];
        let cfg = GAConfig::new(
            6,
            5,
            3,
            0.9,
            0.8,
            0.05,
            genes.clone(),
            CrossoverType::Uniform,
            MutationType::BitFlip,
        );
        let bits_per_gene = cfg.bits(); // global width
        let num_genes = genes.len();

        let ga = GeneticAlgorithm::new(cfg);
        assert_eq!(ga.population().len(), 6);

        for chrom in ga.population() {
            assert_eq!(chrom.genes().len(), num_genes * bits_per_gene);
        }
    }
    #[test]
    fn uniform_crossover_child_bits_are_from_parents() {
        let p1 = Chromosome::new(vec![true, true, true, true, true]);
        let p2 = Chromosome::new(vec![false, false, false, false, false]);

        let child = GeneticAlgorithm::crossover(&p1, &p2, CrossoverType::Uniform);
        assert_eq!(child.genes().len(), 5);

        for (i, &b) in child.genes().iter().enumerate() {
            // must equal either parent's bit at that position
            assert!(b == p1.genes()[i] || b == p2.genes()[i]);
        }
    }

    #[test]
    fn onepoint_and_twopoint_crossover_lengths_match() {
        let p1 = Chromosome::new(vec![true; 16]);
        let p2 = Chromosome::new(vec![false; 16]);

        let c1 = GeneticAlgorithm::crossover(&p1, &p2, CrossoverType::OnePoint);
        let c2 = GeneticAlgorithm::crossover(&p1, &p2, CrossoverType::TwoPoint);

        assert_eq!(c1.genes().len(), 16);
        assert_eq!(c2.genes().len(), 16);
    }
    #[test]
    fn bitflip_mutation_with_prob_1_flips_all_bits() {
        let mut c = Chromosome::new(vec![true, false, true, false, true, false]);
        GeneticAlgorithm::mutate(&mut c, MutationType::BitFlip, 1.0);
        assert_eq!(c.genes(), &vec![false, true, false, true, false, true]);
    }

    #[test]
    fn swap_mutation_preserves_multiset() {
        let mut c = Chromosome::new(vec![true, false, false, true, true, false]);
        let before = c.genes.clone();
        GeneticAlgorithm::mutate(&mut c, MutationType::Swap, 1.0);

        // same length
        assert_eq!(c.genes().len(), before.len());

        // same counts of true/false
        let cnt = |v: &[bool], b: bool| v.iter().filter(|&&x| x == b).count();
        assert_eq!(cnt(&c.genes, true), cnt(&before, true));
        assert_eq!(cnt(&c.genes, false), cnt(&before, false));
    }

    #[test]
    fn scramble_mutation_preserves_multiset() {
        let mut c = Chromosome::new(vec![true, false, true, true, false, false, true]);
        let before = c.genes.clone();
        GeneticAlgorithm::mutate(&mut c, MutationType::Scramble, 1.0);

        // same length and same multiset of bits
        let cnt = |v: &[bool], b: bool| v.iter().filter(|&&x| x == b).count();
        assert_eq!(c.genes().len(), before.len());
        assert_eq!(cnt(&c.genes, true), cnt(&before, true));
        assert_eq!(cnt(&c.genes, false), cnt(&before, false));
    }
    #[test]
    fn step_with_fitness_advances_generation_and_keeps_size() {
        let genes = vec![
            GeneConfig::new(-3, 3, HashSet::new()),
            GeneConfig::new(-9, 9, HashSet::new()),
        ];
        let cfg = GAConfig::new(
            10,
            20,
            5,
            0.9,
            0.8,
            0.1,
            genes,
            CrossoverType::Uniform,
            MutationType::BitFlip,
        );
        let mut ga = GeneticAlgorithm::new(cfg);
        let pop_len = ga.population().len();

        let fitness = vec![1.0; pop_len]; // equal fitness
        ga.step_with_fitness(&fitness);

        assert_eq!(ga.population().len(), pop_len);
        assert_eq!(ga.generation(), 1);
    }
}

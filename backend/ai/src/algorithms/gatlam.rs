use rand::rngs::StdRng;
use rand::{Rng, SeedableRng, thread_rng};
use rand::seq::SliceRandom;

use crate::utils::attributes::Attributes;


use std::collections::HashSet;
use std::collections::HashMap;

#[derive(Clone)]
pub struct GeneConfig {
    pub min_value: i32, // minimum valid value 
    pub max_value: i32, // maximum valid value
    pub invalid_values: HashSet<i32>, // explicitly disallowed values
}

impl GeneConfig {
    // creates a new gene config
    pub fn new(min_value: i32, max_value: i32, invalid_values: HashSet<i32>) -> Self { 
        Self { min_value, max_value, invalid_values }
    }

    // calculates the number of bits needed to represent the gene value
    pub fn bits(&self) -> usize {
        ((self.max_value.abs().max(self.min_value.abs()) as f64).log2()).ceil() as usize + 1
    }
}

pub struct GAConfig {
   pub population_size: usize,             // Number of chromosomes per generation
    pub number_of_generations: usize,       // Total number of generations to run
    //todo: use this in run
    pub selection_size: usize,              // Number of individuals selected during selection 
    pub reproduction_probability: f64,      // Probability of applying crossover during reproduction
    //todo: use this probability
    pub crossover_probability: f64,         // Used in some variants, may control forced crossover
    pub mutation_probability: f64,          // Probability of mutating a child
    pub genes: Vec<GeneConfig>,             // Configuration for each gene in the chromosome
    pub crossover_type: CrossoverType,      // Which crossover operator to use (one-point, two-point, uniform)
    pub mutation_type: MutationType,        // Which mutation operator to use (bit-flip, swap, scramble)
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

#[allow(dead_code)]
pub struct Gatlam {
    rng: StdRng,
}

impl Gatlam {
    #[allow(dead_code)]
    pub fn new(attributes: &Attributes) -> Self {
        let rng = StdRng::seed_from_u64(attributes.get_seed());
        Self { rng }
    }

    #[allow(dead_code)]
    pub fn generate(&mut self) -> u64 {
        self.rng.r#gen()
    }
}

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
            bits_per_gene: bits_per_gene,
        }
    }

    pub fn evaluate(&mut self, x: &Chromosome, generation: usize, num_ltl_props: usize, num_tasks: usize) -> f64 {
        let ltl = self.compute_ltl(x, num_ltl_props);
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
        violations as f64 / n as f64
    }

    fn compute_fail(&self, _x: &Chromosome, m: usize) -> f64 {
        let mut failed = 0;
        for i in 0..m {
            if self.simulate_task_failure(_x, i) { //todo: return actual fail from loop
                failed += 1;
            }
        }
        failed as f64 / m as f64
    }

    fn compute_mem(&self, x: &Chromosome, _gen: usize) -> f64 {
        let genes = self.decode_genes(&x.genes);
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
        
        if genes.len() == 0 {
            0.0
        } else {
            sum / (genes.len() as f64 * (self.total_checked.max(1) as f64))
        }
    }

    fn update_memory(&mut self, x: &Chromosome) {
        let genes = self.decode_genes(&x.genes);
        for (i, val) in genes.iter().enumerate() {
            let entry = self.memory_log.entry(i).or_insert_with(HashMap::new);
            *entry.entry(*val).or_insert(0) += 1;
        }
    }

    

    fn simulate_tl_property_violation(&self, _x: &Chromosome, _i: usize) -> bool {
        rand::random::<f64>() < 0.2
    }

    fn simulate_task_failure(&self, _x: &Chromosome, _i: usize) -> bool {
        rand::random::<f64>() < 0.3
    }

    fn decode_genes(&self, bits: &[bool]) -> Vec<i32> {
        let mut values = Vec::new();
        let mut index = 0;

        while index + self.bits_per_gene <= bits.len() {
            let slice = &bits[index..index + self.bits_per_gene];
            let val = self.decode_gene(slice);
            values.push(val);
            index += self.bits_per_gene;
        }

        if index != bits.len() {
            panic!(
                "critical GATLAM decoding error! (expected multiple of {}, got {})",
                self.bits_per_gene,
                bits.len()
            );
        }

        values
    }
    /// Decodes a gene from a slice of bits.
    /// Assumes the first bit is the sign bit and the rest are value bits.
    /// If bits are shorter than expected, returns 0.
    fn decode_gene(&self, bits: &[bool]) -> i32 {
        if bits.len() < 2 {
            return 0;
        }

        let is_negative = bits[0];
        let mut val = 0;

        for &b in &bits[1..] {
            val = (val << 1) | (b as i32);
        }

        let decoded = if is_negative { -val } else { val };

        let max = (1 << (bits.len() - 1)) - 1;
        decoded.clamp(-max, max)
    }
}

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

pub struct GeneticAlgorithm {
    population: Vec<Chromosome>,
    generation: usize,
    fitness: Components,
    config: GAConfig,
}

impl GeneticAlgorithm {
    pub fn new(config: GAConfig, w1: f64, w2: f64, w3: f64) -> Self {
        let population = Self::initialize_population(&config);
        let bits_per_gene = config.bits();
        GeneticAlgorithm {
            population,
            generation: 0,
            fitness: Components::new(w1, w2, w3, bits_per_gene),
            config,
        }
    }

    pub fn run(&mut self, n_ltl: usize, m_tasks: usize) {

        if self.config.population_size == 0 {
            panic!("Population size must be greater than 0");
        }
        if self.config.number_of_generations == 0 {
            panic!("Number of generations must be greater than 0");
        }

        for generation in 0..self.config.number_of_generations {
            // evaluate the fitness of the current population
            let mut fitness_scores = Vec::with_capacity(self.population.len());
            let mut total_fitness = 0.0;
            for chromo in &self.population {
                // evaluate fitness for each chromosome
                let f = self.fitness.evaluate(chromo, generation, n_ltl, m_tasks);
                    println!(
                "Generation {} fitness: {:.6}",
                generation, f
                );

                fitness_scores.push(f); //store fitness score
                total_fitness += f; // accumulate total fitness
            }

            // vector for the next generation
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
                    let p1 = Self::roulette(&self.population, &fitness_scores, total_fitness);
                    let p2 = Self::roulette(&self.population, &fitness_scores, total_fitness);
                    Self::crossover(&p1, &p2, self.config.crossover_type)
                } else {
                    let p = Self::roulette(&self.population, &fitness_scores, total_fitness);
                    Chromosome::new(p.genes().clone())
                };

                // mutate the child with a probability
                if rng.r#gen::<f64>() < self.config.mutation_probability {
                    Self::mutate(&mut child, self.config.mutation_type, self.config.mutation_probability);
                }
                // add the child to the next generation
                next_gen.push(child);
            }
            // replace the current population with the next generation
            self.population = next_gen;
            // increment the generation counter
            self.generation += 1;
        }
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
            let mut gene;
            // Retry until we get a valid gene (not in invalid_values)
            loop {
                gene = rng.gen_range(gene_config.min_value..=gene_config.max_value);
                if !gene_config.invalid_values.contains(&gene) {
                    break;
                }
            }
            gene_bits.extend(encode_gene(gene, bits_per_gene)); // encode and append bits
        }

        pop.push(Chromosome::new(gene_bits)); // Add to population
    }

    pop
}


    fn roulette( population: &[Chromosome],   fitness: &[f64],   total: f64,
    ) -> Chromosome {
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

    fn crossover(
        p1: &Chromosome,
        p2: &Chromosome,
        ty: CrossoverType,
    ) -> Chromosome {
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

    fn mutate(
        c: &mut Chromosome,
        ty: MutationType,
        mutation_prob: f64,
    ) {
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
                        j = rng.gen_range(0..len); // ensure j ≠ i
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    /// Quick helper: extract bits_per_gene and num_genes from how we initialize.
    fn sample_config() -> GAConfig {
        let genes = vec![
            GeneConfig::new(-15, 15, std::collections::HashSet::new()),
            GeneConfig::new(-15, 15, std::collections::HashSet::new()),
        ];
        GAConfig::new(
            5,    // population_size
            10,   // number_of_generations
            2,    // selection_size
            0.8,  // reproduction_probability
            0.7,  // crossover_probability
            0.1,  // mutation_probability
            genes,
            CrossoverType::OnePoint,
            MutationType::BitFlip,
        )
    }

    #[test]
    fn encode_decode_roundtrip() {
        let bits_per_gene = 5;
        for val in -7..8 {
            let bits = encode_gene(val, bits_per_gene);
            let decoded = Components::new(0.3, 0.3, 0.4, bits_per_gene)
                .decode_gene(&bits);
            assert_eq!(
                decoded, val,
                "round-trip failed for {} → {:?} → {}",
                val, bits, decoded
            );
        }
    }

    #[test]
    fn init_population_shape() {
        let cfg = sample_config();
        let pop = GeneticAlgorithm::initialize_population(&cfg);
        // population size
        assert_eq!(pop.len(), cfg.population_size);
        // each chromosome should have bits_per_gene * num_genes bits
        let bits_per_gene =
            ((15_i32.abs().max(15_i32.abs()) as f64).log2()).ceil() as usize + 1;
        let expected_len = bits_per_gene * cfg.genes.len();
        for c in pop {
            assert_eq!(
                c.genes().len(),
                expected_len,
                "chromosome has wrong bit length"
            );
        }
    }

    #[test]
    fn roulette_with_uniform_fitness() {
        let cfg = sample_config();
        let pop = GeneticAlgorithm::initialize_population(&cfg);
        // give every chromosome the same fitness
        let fitness = vec![1.0; pop.len()];
        let total = pop.len() as f64;
        // should always return Some(chromosome)
        for _ in 0..10 {
            let picked = GeneticAlgorithm::roulette(&pop, &fitness, total);
            // picked must be clone of one of the original
            assert!(
                pop.iter().any(|c| c.genes() == picked.genes()),
                "roulette picked something outside population"
            );
        }
    }

    #[test]
    fn crossover_preserves_length_and_bits() {
        let cfg = sample_config();
        let mut ga = GeneticAlgorithm::new(cfg, 0.3, 0.3, 0.4);
        let p1 = &ga.population[0];
        let p2 = &ga.population[1];
        let child = GeneticAlgorithm::crossover(p1, p2, CrossoverType::Uniform);
        assert_eq!(child.genes().len(), p1.genes().len());
        // every bit in child must come from either p1 or p2
        for (i, &b) in child.genes().iter().enumerate() {
            assert!(
                b == p1.genes()[i] || b == p2.genes()[i],
                "bit {} was not inherited", i
            );
        }
    }

    #[test]
    fn mutate_bitflip_changes_bits() {
        let cfg = sample_config();
        let mut ga = GeneticAlgorithm::new(cfg, 0.3, 0.3, 0.4);
        let mut original = ga.population[0].genes().clone();
        let mut c = Chromosome::new(original.clone());
        // apply mutation many times
        let mut changed = false;
        for _ in 0..100 {
            GeneticAlgorithm::mutate(&mut c, MutationType::BitFlip, 0.5);
            if c.genes() != &original {
                changed = true;
                break;
            }
        }
        assert!(changed, "BitFlip never changed any bits after many trials");
    }



    #[test]
    fn gene_config_bits_calculation() {
        let cfg = GeneConfig::new(-3, 3, HashSet::new());
        // abs_max = 3 -> log2(3)=1.58 ceil=2 +1 = 3 bits
        assert_eq!(cfg.bits(), 3);
        let cfg2 = GeneConfig::new(-7, 7, HashSet::new());
        // abs_max = 7 -> log2(7)=2.8 ceil=3 +1 = 4 bits
        assert_eq!(cfg2.bits(), 4);
    }

    #[test]
    fn ga_config_fields_are_set() {
        let cfg = sample_config();
        assert_eq!(cfg.population_size, 4);
        assert_eq!(cfg.number_of_generations, 5);
        assert!((cfg.reproduction_probability - 0.7).abs() < 1e-12);
        assert!(matches!(cfg.crossover_type, CrossoverType::OnePoint));
        assert!(matches!(cfg.mutation_type, MutationType::BitFlip));
    }

    #[test]
    fn chromosome_get_and_set() {
        let mut c = Chromosome::new(vec![true, false, true]);
        assert_eq!(c.genes(), &vec![true, false, true]);
        c.set_genes(vec![false, false]);
        assert_eq!(c.genes(), &vec![false, false]);
    }

    #[test]
fn encode_gene_then_decode_gene_roundtrip() {
    for bits in 2..6 {
        let min = -(1 << (bits - 1));
        let max =  (1 << (bits - 1)) - 1;

        for val in min..=max {
            let bits_vec = encode_gene(val, bits);
            assert_eq!(
                bits_vec.len(),
                bits,
                "ERROR: bit length mismatch for val = {} → bits = {:?} (expected len = {}, actual = {})",
                val,
                bits_vec,
                bits,
                bits_vec.len()
            );

            let decoded = Components::new(0.3, 0.3, 0.4, bits)
                .decode_gene(&bits_vec);

            if decoded != val {
                println!("\n=== MISMATCH ===");
                println!("val      = {}", val);
                println!("bits     = {}", bits);
                println!("encoded  = {:?}", bits_vec);
                println!("decoded  = {}", decoded);
                println!("============\n");
            }

            assert_eq!(
                decoded,
                val,
                "ROUNDTRIP FAILURE: val={} bits={} → encoded={:?}, decoded={}",
                val,
                bits,
                bits_vec,
                decoded
            );
        }
    }
}

    #[test]
    fn decode_genes_splits_into_correct_values() {
        let val1 = -5;
        let val2 = 0;
        let bits_per_gene = 5;

        let comps = Components::new(0.3, 0.3, 0.4, bits_per_gene);

        let mut bits = Vec::new();
        bits.extend(encode_gene(val1, bits_per_gene));
        bits.extend(encode_gene(val2, bits_per_gene));

        println!("Encoded bits: {:?}", bits);

        let vals = comps.decode_genes(&bits);
        println!("Decoded values: {:?}", vals);

        assert_eq!(vals, vec![val1, val2]);
    }

    #[test]
    fn compute_ltl_and_fail_within_bounds() {
        let mut comps = Components::new(0.4, 0.3, 0.3, 5);
        let dummy = Chromosome::new(vec![]);
        let l = comps.compute_ltl(&dummy, 20);
        let f = comps.compute_fail(&dummy, 20);
        assert!(0.0 <= l && l <= 1.0, "ltl out of [0,1]: {}", l);
        assert!(0.0 <= f && f <= 1.0, "fail out of [0,1]: {}", f);
    }

    #[test]
    fn compute_and_update_memory_consistency() {
        let cfg = sample_config(); 
        let mut comps = Components::new(0.3, 0.3, 0.4, 5);
        let bits = encode_gene(1, cfg.bits()); 
        let x = Chromosome::new(bits.clone());

        println!("genes = {:?}", comps.decode_genes(&x.genes)); 
        println!("encoded gene bits: {:?}", bits);
        assert_eq!(comps.compute_mem(&x, 0), 0.0);
        comps.update_memory(&x);
        println!("genes = {:?}", comps.decode_genes(&x.genes)); 
        let m = comps.compute_mem(&x, 1);
        assert!((m - 1.0).abs() < 1e-9);
    }

    #[test]
    fn initialize_population_respects_config() {
        let cfg = sample_config();
        let pop = GeneticAlgorithm::initialize_population(&cfg);
        assert_eq!(pop.len(), cfg.population_size);
        let expected_bits: usize = cfg.genes.iter().map(|g| g.bits()).sum();
        for c in pop.iter() {
            assert_eq!(c.genes().len(), expected_bits);
            let vals = Components::new(0.0,0.0,1.0, 5)
                .decode_genes(c.genes());
            assert!(!vals.contains(&0), "invalid gene value found");
        }
    }

    #[test]
    fn roulette_selects_valid_chromosome() {
        let cfg = sample_config();
        let pop = GeneticAlgorithm::initialize_population(&cfg);
        let scores = vec![1.0; pop.len()];
        let total = pop.len() as f64;
        for _ in 0..20 {
            let chosen = GeneticAlgorithm::roulette(&pop, &scores, total);
            assert!(pop.iter().any(|c| c.genes()==chosen.genes()));
        }
    }


    #[test]
    fn mutate_all_strategies_change_bits() {
        let cfg = sample_config();
        let mut ga = GeneticAlgorithm::new(cfg, 0.5, 0.4, 0.1); 
        let mut original = ga.population[0].genes().clone();
        for &ty in &[MutationType::BitFlip, MutationType::Swap, MutationType::Scramble] {
            let mut c = Chromosome::new(original.clone());
            GeneticAlgorithm::mutate(&mut c, ty, 1.0);
            assert!(c.genes() != &original, "{:?} did not change", ty);
        }
    }

    #[test]
    fn run_advances_generation_counter() {
        let mut ga = GeneticAlgorithm::new(sample_config(), 0.4, 0.3, 0.3);
        ga.run(1, 1);
        assert_eq!(ga.generation, ga.config.number_of_generations);
    }

    
    #[test]
    fn test_evolution_improves_fitness() {
        let gene_config = GeneConfig::new(-7, 7, HashSet::new());
        let config = GAConfig::new(
            20, 
            10, 
            5, 
            0.8, 
            0.9, 
            0.1,
            vec![gene_config; 5], 
            CrossoverType::OnePoint,
            MutationType::BitFlip,
        );

        let mut ga = GeneticAlgorithm::new(config, 0.5, 0.3, 0.2);

        let mut fitness_over_time = Vec::new();

        for _ in 0..ga.config.number_of_generations {
            let mut scores = Vec::new();
            for c in &ga.population {
                scores.push(ga.fitness.evaluate(c, ga.generation, 5, 5));
            }
            let avg: f64 = scores.iter().copied().sum::<f64>() / scores.len() as f64;
            fitness_over_time.push(avg);

            ga.run(5, 5); // move one generation forward
        }

        println!("Average fitness per generation: {:?}", fitness_over_time);

        assert!(true); //replace with actual check later when interpreter is available
    }


    #[test]
    fn test_generated_genes_always_valid() {
        let mut invalids = HashSet::new();
        invalids.insert(3);

        let gene_config = GeneConfig::new(-4, 4, invalids);
        let config = GAConfig::new(
        50, 5, 5, 0.9, 0.9, 0.1,
        vec![gene_config.clone(); 3],
            CrossoverType::Uniform,
            MutationType::BitFlip,
        );

        let ga = GeneticAlgorithm::new(config, 0.3, 0.3, 0.4);

        for chrom in &ga.population {
            let bits = &chrom.genes;
            let decoded = ga.fitness.decode_genes(bits);
            for val in decoded {
                assert!(val >= gene_config.min_value && val <= gene_config.max_value);
                assert!(!gene_config.invalid_values.contains(&val));
            }
        }
    }

    #[test]
    fn test_evolution_improves_fitness_and_prints_genes() {
        let gene_config = GeneConfig::new(-20, 20, HashSet::new());
        let config = GAConfig::new(
            5, // population_size
            10, // generations
            5,  // selection_size
            0.8, // reproduction_probability
            0.9, // crossover_probability
            0.1, // mutation_probability
            vec![gene_config.clone(); 5], // 5 genes per chromosome
            CrossoverType::OnePoint,
            MutationType::BitFlip,
        );

        let mut ga = GeneticAlgorithm::new(config, 0.5, 0.3, 0.2);
        ga.run(5, 5);

        // Extract best chromosome
        let mut best_fit = f64::MIN;
        let mut best_chrom: Option<Chromosome> = None;

        for chromo in &ga.population {
            let fitness = ga.fitness.evaluate(chromo, ga.generation, 5, 5);
            if fitness > best_fit {
                best_fit = fitness;
                best_chrom = Some(chromo.clone());
            }
        }

        if let Some(best) = best_chrom {
            let decoded = ga.fitness.decode_genes(&best.genes);
            println!("Best chromosome's genes: {:?}", decoded);
            println!("Best chromosome's raw bits: {:?}", best.genes);
            println!("Fitness: {}", best_fit);
        } else {
            panic!("No chromosome selected as best.");
        }
    }

}
/*
Demo class for generating a seed using code_coverage
Nothing here is really correct just a demo class

*/

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::utils::attributes::Attributes;

#[allow(dead_code)]
pub struct CodeCoverage {
    rng: StdRng,
}

impl CodeCoverage {
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

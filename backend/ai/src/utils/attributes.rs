/*
Demo class that can be used to store the attributes that the AI wil need

For example the seed to generate the inital AI, the length and so on

All of the current attributes are just there for show - change all of them

*/
pub struct Attributes {
    seed: u64,
    length: usize,
}

impl Attributes {
    // Constructor
    pub fn new(seed: u64, length: usize) -> Self {
        Self { seed, length }
    }

    // Setters
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    pub fn set_length(&mut self, length: usize) {
        self.length = length;
    }

    // Getters
    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    pub fn get_length(&self) -> usize {
        self.length
    }
}

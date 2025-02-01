/// LCG based pseudo-random number generator.
#[derive(Default)]
pub struct RandomNumberGenerator {
    seed: i32,
    n: i32,
}

impl RandomNumberGenerator {
    pub fn rand(&mut self) -> i32 {
        // x[n+1] = (a * x[n] + c) % m
        // where m = i32::MAX
        self.n = 1103515245_i32.wrapping_mul(self.n.wrapping_add(12345));
        self.n
    }

    pub fn set_seed(&mut self, seed: i32) {
        self.seed = seed;
        self.n = seed;
    }

    pub fn seed(&self) -> i32 {
        self.seed
    }
}

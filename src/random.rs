const MULTIPLIER: u32 = 214013;
const INCREMENT: u32 = 2541011;
pub const RANGE: u32 = 0x7fff;
const SEED: u32 = 0x12345678;

#[derive(Clone, Copy, Debug)]
pub struct Rand {
    val: u32,
}

impl Rand {
    pub fn new() -> Self {
        Rand { val: SEED }
    }
    pub fn next(&mut self) -> u32 {
        self.val = MULTIPLIER * self.val + INCREMENT;
        (self.val >> 16) & RANGE
    }
    pub fn set_seed(&mut self, seed: u32) {
        self.val = seed;
    }
}

use rand::distributions::uniform::SampleUniform;
use rand::distributions::{Distribution, Standard};
use rand::prng::XorShiftRng;
use rand::Rng;

pub struct ProgramContext {
    rng: XorShiftRng,
}

impl ProgramContext {

    pub fn new(rng: XorShiftRng) -> ProgramContext {
        ProgramContext { rng }
    }

    #[allow(dead_code)]
    pub fn gen_value<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        self.rng.gen()
    }

    pub fn gen_range<T: PartialOrd + SampleUniform>(&mut self, min_inclusive: T, max_exclusive: T) -> T {
        self.rng.gen_range(min_inclusive, max_exclusive)
    }

}

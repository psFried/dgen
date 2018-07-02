use super::{Generator, DataGenRng};
use rand::Rng;
use std::fmt::{self, Display};

pub struct NullableGenerator<T: Display> {
    wrapped_generator: Box<Generator<Output=T>>,
    null_frequency: Box<Generator<Output=f64>>,
}

impl <T: Display> NullableGenerator<T> {
    pub fn new(wrapped_generator: Box<Generator<Output=T>>, null_frequency: Box<Generator<Output=f64>>) -> NullableGenerator<T> {
        NullableGenerator {
            wrapped_generator,
            null_frequency,
        }
    }
}

impl <T: Display> Generator for NullableGenerator<T> {
    type Output = T;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Option<&T> {
        let NullableGenerator {ref mut wrapped_generator, ref mut null_frequency} = *self;
        let frequency = null_frequency.gen_value(rng);
        let gen_null = rng.gen_bool(frequency.cloned().unwrap_or(100.0));
        if gen_null {
            None
        } else {
            wrapped_generator.gen_value(rng)
        }
    }
}

impl <T: Display> Display for NullableGenerator<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let NullableGenerator {ref wrapped_generator, ref null_frequency} = *self;

        write!(f, "nullable({}, {})", wrapped_generator, null_frequency)
    }
}


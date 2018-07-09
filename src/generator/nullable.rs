use super::{Generator, DataGenRng};
use writer::DataGenOutput;
use rand::Rng;
use std::fmt::{self, Display};
use std::io;

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

    fn will_generate(&mut self, rng: &mut DataGenRng) -> bool {
        let frequency = self.null_frequency.gen_value(rng);
        rng.gen_bool(frequency.cloned().unwrap_or(100.0))
    }
}

impl <T: Display + 'static> Generator for NullableGenerator<T> {
    type Output = T;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Option<&T> {
        if self.will_generate(rng) {
            self.wrapped_generator.gen_value(rng)
        } else {
            None
        }
    }

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> io::Result<u64> {
        if self.will_generate(rng) {
            self.wrapped_generator.write_value(rng, output)
        } else {
            Ok(0)
        }
    }

    fn new_from_prototype(&self) -> Box<Generator<Output=T>> {
        let wrapped_generator = self.wrapped_generator.new_from_prototype();
        let null_frequency = self.null_frequency.new_from_prototype();
        Box::new(NullableGenerator{ wrapped_generator, null_frequency })
    }
}

impl <T: Display> Display for NullableGenerator<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let NullableGenerator {ref wrapped_generator, ref null_frequency} = *self;

        write!(f, "nullable({}, {})", wrapped_generator, null_frequency)
    }
}


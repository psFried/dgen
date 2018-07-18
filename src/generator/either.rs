use generator::{Generator, DataGenRng, DynDecimalGenerator, DynGenerator};
use writer::DataGenOutput;

use std::fmt::{self, Display};
use rand::Rng;
use failure::Error;

pub const EITHER_FUNCTION_NAME: &'static str = "select2";
pub const MAX_FREQUENCY: f64 = 1.0;
pub const MIN_FREQUENCY: f64 = 0.0;

pub struct Either<T: Display + Clone + 'static> {
    a: DynGenerator<T>,
    b: DynGenerator<T>,
    a_frequency: DynDecimalGenerator,
}

impl <T: Display + Clone + 'static> Either<T> {
    pub fn new(a_frequency: DynDecimalGenerator, a: DynGenerator<T>, b: DynGenerator<T>) -> DynGenerator<T> {
        Box::new(Either { a_frequency, a, b })
    }

    pub fn with_even_odds(a: DynGenerator<T>, b: DynGenerator<T>) -> DynGenerator<T> {
        let freq = ::generator::constant::ConstantGenerator::create(0.5);
        Either::new(freq, a, b)
    }

    fn will_select_a(&mut self, rng: &mut DataGenRng) -> Result<bool, Error> {
        let a_freq = self.a_frequency.gen_value(rng)?.cloned().unwrap_or(50.0);
        let a_freq = a_freq.min(MAX_FREQUENCY).max(MIN_FREQUENCY);
        Ok(rng.gen_bool(a_freq))
    }
}


impl <T: Display + Clone + 'static> Generator for Either<T> {
    type Output = T;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&T>, Error> {
        if self.will_select_a(rng)? {
            self.a.gen_value(rng)
        } else {
            self.b.gen_value(rng)
        }
    }

    fn write_value(&mut self, rng: &mut DataGenRng, out: &mut DataGenOutput) -> Result<u64, Error> {
        if self.will_select_a(rng)? {
            self.a.write_value(rng, out)
        } else {
            self.b.write_value(rng, out)
        }
    }

    fn new_from_prototype(&self) -> DynGenerator<T> {
        let a_frequency = self.a_frequency.new_from_prototype();
        let a = self.a.new_from_prototype();
        let b = self.b.new_from_prototype();
        Box::new(Either {a_frequency, a, b})
    }
}

impl <T: Display + Clone + 'static> Display for Either<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({}, {}, {})", EITHER_FUNCTION_NAME, self.a_frequency, self.a, self.b)
    }
}



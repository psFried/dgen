use super::{Generator, DataGenRng};
use writer::DataGenOutput;
use rand::Rng;
use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::io;


pub struct OneOfGenerator<T: Display> {
    wrapped: Vec<Box<Generator<Output=T>>>,
    _phantom_data: PhantomData<T>,
}

impl <T: Display + 'static> OneOfGenerator<T> {
    pub fn new(wrapped: Vec<Box<Generator<Output=T>>>) -> Box<Generator<Output=T>> {
        Box::new(OneOfGenerator {
            wrapped,
            _phantom_data: PhantomData,
        })
    }
}

impl <T: Display> Generator for OneOfGenerator<T> {
    type Output = T;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Option<&T> {
        let gen = rng.choose_mut(self.wrapped.as_mut_slice());
        gen.and_then(|g| {
            g.gen_value(rng)
        })
    }

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> io::Result<u64> {
        let gen = rng.choose_mut(self.wrapped.as_mut_slice());
        // gen will be None only if `wrapped` is an empty vec
        gen.map(|g| {
            g.write_value(rng, output)
        }).unwrap_or(Ok(0))
    }
}

impl <T: Display> Display for OneOfGenerator<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("oneOf(")?;
        let mut first = true;
        for (idx, gen) in self.wrapped.iter().enumerate() {
            if !first {
                f.write_str(", ")?;
            } else {
                first = false;
            }
            write!(f, "{}", gen)?;
        }
        f.write_str(")")
    }
}

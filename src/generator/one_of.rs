use super::{Generator, DataGenRng};
use rand::Rng;
use std::fmt::{self, Display};
use std::marker::PhantomData;


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

use super::{Generator, DataGenRng};
use writer::DataGenOutput;
use rand::Rng;
use std::fmt::{self, Display};
use std::marker::PhantomData;
use failure::Error;


pub struct OneOfGenerator<T: Display + ?Sized + 'static> {
    wrapped: Vec<Box<Generator<Output=T>>>,
    _phantom_data: PhantomData<T>,
}

impl <T: Display + ?Sized + 'static> OneOfGenerator<T> {
    pub fn new(wrapped: Vec<Box<Generator<Output=T>>>) -> Box<Generator<Output=T>> {
        Box::new(OneOfGenerator {
            wrapped,
            _phantom_data: PhantomData,
        })
    }
}

impl <T: Display + ?Sized + 'static> Generator for OneOfGenerator<T> {
    type Output = T;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&T>, Error> {
        match rng.choose_mut(self.wrapped.as_mut_slice()) {
            Some(gen) => gen.gen_value(rng),
            None => Ok(None)
        }
    }

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> Result<u64, Error> {
        let gen = rng.choose_mut(self.wrapped.as_mut_slice());
        // gen will be None only if `wrapped` is an empty vec
        match gen {
            Some(g) => g.write_value(rng, output),
            None => Ok(0)
        }
    }
    
    fn new_from_prototype(&self) -> Box<Generator<Output=T>> {
        let wrapped = self.wrapped.iter().map(|g| g.new_from_prototype()).collect::<Vec<Box<Generator<Output=T>>>>();
        Box::new(OneOfGenerator {wrapped, _phantom_data: PhantomData})
    }
}

impl <T: Display + ?Sized + 'static> Display for OneOfGenerator<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("oneOf(")?;
        for (idx, gen) in self.wrapped.iter().enumerate() {
            if idx > 0 {
                f.write_str(", ")?;
            }  
            write!(f, "{}", gen)?;
        }
        f.write_str(")")
    }
}

use generator::{Generator, DataGenRng, DynGenerator};
use writer::DataGenOutput;
use rand::Rng;

use std::fmt::{self, Display};
use failure::Error;

pub const STABLE_SELECT_FUN_NAME: &'static str = "stable_select";

pub struct StableSelect<T: Display + ?Sized + 'static> {
    wrapped: Vec<DynGenerator<T>>,
    index: Option<usize>,
}

impl <T: Display + ?Sized + 'static> StableSelect<T> {
    pub fn create(wrapped: Vec<DynGenerator<T>>) -> DynGenerator<T> {
        Box::new(StableSelect{
            wrapped,
            index: None
        })
    }

    fn get_gen(&mut self, rng: &mut DataGenRng) -> &mut DynGenerator<T> {
        if self.index.is_none() {
            let i = rng.gen_range(0, self.wrapped.len());
            self.index = Some(i);
        }
        let i = self.index.unwrap();
        self.wrapped.get_mut(i).unwrap()
    }
}

impl <T: Display + ?Sized + 'static> Generator for StableSelect<T> {
    type Output = T;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&T>, Error> {
        let gen = self.get_gen(rng);
        gen.gen_value(rng)
    }

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> Result<u64, Error> {
        let gen = self.get_gen(rng);
        gen.write_value(rng, output)
    }

    fn new_from_prototype(&self) -> DynGenerator<T> {
        let new_wrapped = self.wrapped.iter().map(|g| g.new_from_prototype()).collect();
        StableSelect::create(new_wrapped)
    }
}

impl <T: Display + ?Sized + 'static> Display for StableSelect<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", STABLE_SELECT_FUN_NAME)?;
        let mut first = true;
        for gen in self.wrapped.iter() {
            if !first {
                f.write_str(", ")?;
                first = false;
            }
            write!(f, "{}", gen)?;
        }
        f.write_str(")")
    }
}
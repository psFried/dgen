use super::ColumnGenerator;
use rand::prelude::{Rng, Distribution};
use rand::distributions::Alphanumeric;
use formatter::Formatter;
use std::io;
use std::marker::PhantomData;


pub struct OneOfGenerator {
    values: Vec<String>,
}

impl OneOfGenerator {
    pub fn new(values: Vec<String>) -> OneOfGenerator {
        OneOfGenerator {values}
    }
}

impl ColumnGenerator for OneOfGenerator {
    fn gen_value<R: Rng, F: Formatter>(&mut self, rng: &mut R, formatter: &mut F) -> io::Result<()> {
        let value = rng.choose(self.values.as_slice());

        if let Some(string) = value {
            formatter.write_str(string)
        } else {
            // if values is empty, then we'll just always write null
            formatter.write_null()
        }
    }
}



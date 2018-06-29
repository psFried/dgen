use super::ColumnGenerator;
use rand::Rng;
use std::fmt::Display;
use std::io;
use std::marker::PhantomData;
use formatter::Formatter;

pub struct NullableGenerator<S: ColumnGenerator> {
    wrapped_generator: S,
    null_frequency: f64,
}

impl <S: ColumnGenerator> NullableGenerator<S> {
    pub fn new(wrapped_generator: S, null_frequency: f64) -> NullableGenerator<S> {
        NullableGenerator {
            wrapped_generator,
            null_frequency,
        }
    }
}


impl <S: ColumnGenerator> ColumnGenerator for NullableGenerator<S> {
    fn gen_value<R: Rng, F: Formatter>(&mut self, rng: &mut R, formatter: &mut F) -> io::Result<()> {
        if rng.gen_bool(self.null_frequency) {
            formatter.write_null()
        } else {
            self.wrapped_generator.gen_value(rng, formatter)
        }
    }
}
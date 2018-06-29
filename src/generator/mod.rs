mod nullable;
mod string;
mod one_of;

use formatter::Formatter;
use std::fmt::{Display, Write};
use std::io;
use rand::Rng;
use std::marker::PhantomData;

pub trait ColumnGenerator {
    fn gen_value<R: Rng, F: Formatter>(&mut self, rng: &mut R, formatter: &mut F) -> io::Result<()>;
}

pub struct ConstantGenerator<T: Display> {
    value: T,
    buffer: String,
}
impl <T: Display> ColumnGenerator for ConstantGenerator<T> {
    fn gen_value<R: Rng, F: Formatter>(&mut self, rng: &mut R, formatter: &mut F) -> io::Result<()> {
        let ConstantGenerator {ref value, ref mut buffer} = *self;
        buffer.clear();
        write!(buffer, "{}", value).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("{}", e))
        }).and_then(|_| {
            formatter.write_str(buffer.as_str())
        })
    }
}


pub struct Column<S: ColumnGenerator> {
    name: String,
    generator: S,
}

impl <S: ColumnGenerator> Column<S> {
    pub fn new(name: String, generator: S) -> Column<S> {
        Column {
            name,
            generator,
        }
    }

    pub fn write_column<R: Rng, F: Formatter>(&mut self, rng: &mut R, formatter: &mut F) -> io::Result<()> {
        formatter.write_column_start(self.name()).and_then(|()| {
            self.generator.gen_value(rng, formatter)
        }).and_then(|()| {
            formatter.write_column_end(self.name())
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

}




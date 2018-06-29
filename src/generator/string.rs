use super::ColumnGenerator;
use rand::prelude::{Rng, Distribution};
use rand::distributions::Alphanumeric;
use formatter::Formatter;
use std::io;
use std::marker::PhantomData;

pub trait StringType {
    fn gen_char<R: Rng>(rng: &mut R) -> char;
}

pub struct Ascii;
impl StringType for Ascii {
    fn gen_char<R: Rng>(rng: &mut R) -> char {
        rng.sample(Alphanumeric)
    }
}


pub struct StringGenerator<T: StringType> {
    min_length: usize,
    max_length: usize,
    buffer: String,
    _phantom_data: PhantomData<T>,
}



impl <T: StringType> StringGenerator<T> {
    pub fn new<S: StringType>(min_length: usize, max_length: usize) -> StringGenerator<S> {
        StringGenerator {
            min_length,
            max_length,
            buffer: String::with_capacity(max_length),
            _phantom_data: PhantomData,
        }
    }
}

impl <T: StringType> ColumnGenerator for StringGenerator<T> {
    fn gen_value<R: Rng, F: Formatter>(&mut self, rng: &mut R, formatter: &mut F) -> io::Result<()> {
        let target_len = rng.gen_range(self.min_length, self.max_length);

        while self.buffer.len() < target_len {
            let next_char = T::gen_char(rng);
            if (next_char.len_utf8() + self.buffer.len()) <= self.max_length {
                self.buffer.push(next_char);
            }
            if self.buffer.len() >= target_len {
                break;
            }
        }

        formatter.write_str(self.buffer.as_str())
    }
}

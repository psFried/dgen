use super::{DataGenRng, DynCharGenerator, DynStringGenerator, DynUnsignedIntGenerator, Generator};
use failure::Error;
use rand::distributions::Alphanumeric;
use rand::prelude::Rng;
use std::fmt::{self, Display};
use writer::DataGenOutput;

#[derive(Clone)]
pub struct AsciiChar(char);

impl AsciiChar {
    pub fn new() -> AsciiChar {
        AsciiChar('x') // initial char doesn't matter, as it's just a tiny buffer that gets reused
    }
}

impl Generator for AsciiChar {
    type Output = char;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&char>, Error> {
        self.0 = rng.sample(Alphanumeric);
        Ok(Some(&self.0))
    }

    fn write_value(
        &mut self,
        rng: &mut DataGenRng,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        if let Some(val) = self.gen_value(rng)? {
            output.write_string(val).map_err(Into::into)
        } else {
            unreachable!()
        }
    }

    fn new_from_prototype(&self) -> Box<Generator<Output = char>> {
        Box::new(self.clone())
    }
}

impl Display for AsciiChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("ascii()")
    }
}

pub struct StringGenerator {
    char_gen: Box<Generator<Output = char>>,
    length_gen: Box<Generator<Output = u64>>,
    buffer: String,
}

impl StringGenerator {
    pub fn with_length(length_gen: DynUnsignedIntGenerator) -> DynStringGenerator {
        StringGenerator::new(length_gen, default_charset())
    }

    pub fn new(
        length_gen: DynUnsignedIntGenerator,
        char_gen: DynCharGenerator,
    ) -> DynStringGenerator {
        Box::new(StringGenerator {
            char_gen,
            length_gen,
            buffer: String::with_capacity(16),
        })
    }

    fn fill_buffer(&mut self, rng: &mut DataGenRng) -> Result<(), Error> {
        self.buffer.clear();
        let remaining_chars = self.length_gen.gen_value(rng)?.cloned().unwrap_or(16);

        for _ in 0..remaining_chars {
            let next_char = self.char_gen.gen_value(rng)?.cloned().unwrap_or('x');
            self.buffer.push(next_char);
        }
        Ok(())
    }
}

pub fn default_charset() -> Box<Generator<Output = char>> {
    Box::new(AsciiChar::new())
}

pub fn default_string_length_generator() -> Box<Generator<Output = u64>> {
    // TODO: replace the default with a range
    Box::new(::generator::constant::ConstantGenerator::new(Some(16)))
}

impl Generator for StringGenerator {
    type Output = str;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&str>, Error> {
        self.fill_buffer(rng)?;
        Ok(Some(&self.buffer))
    }

    fn write_value(
        &mut self,
        rng: &mut DataGenRng,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        self.fill_buffer(rng)?;
        output.write_string(&self.buffer).map_err(Into::into)
    }

    fn new_from_prototype(&self) -> Box<Generator<Output = str>> {
        let char_gen = self.char_gen.new_from_prototype();
        let length_gen = self.length_gen.new_from_prototype();
        let buffer = String::with_capacity(self.buffer.capacity());
        Box::new(StringGenerator {
            char_gen,
            length_gen,
            buffer,
        })
    }
}

impl Display for StringGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "string({}, {})", self.length_gen, self.char_gen)
    }
}

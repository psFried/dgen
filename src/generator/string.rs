use super::{Generator, GeneratorArg, GeneratorType, DataGenRng, DynUnsignedIntGenerator, DynCharGenerator, DynStringGenerator};
use rand::prelude::{Rng, Distribution};
use rand::distributions::Alphanumeric;
use std::marker::PhantomData;
use std::fmt::{self, Display};


pub struct AsciiChar(char);

impl AsciiChar {
    pub fn new() -> AsciiChar {
        AsciiChar('x') // initial char doesn't matter, as it's just a tiny buffer that gets reused
    }
}

impl Generator for AsciiChar {
    type Output = char;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Option<&char> {
        self.0 = rng.sample(Alphanumeric);
        Some(&self.0)
    }
}


impl Display for AsciiChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("ascii()")
    }
}


pub fn char_generator(name: &str) -> Option<Box<Generator<Output=char>>> {
    match name {
        "ascii" => Some(Box::new(AsciiChar::new())),
        _ => None
    }
}

pub struct StringGenerator {
    char_gen: Box<Generator<Output=char>>,
    length_gen: Box<Generator<Output=u64>>,
    buffer: String,
}

const INVALID_STRING_ARGS: &'static str = "Invalid arguments for string generator, expected ([UnsignedInt], [Char])";

impl StringGenerator {
     pub fn with_length(length_gen: DynUnsignedIntGenerator) -> DynStringGenerator {
         StringGenerator::new(length_gen, default_charset())
     }

    pub fn new(length_gen: DynUnsignedIntGenerator, char_gen: DynCharGenerator) -> DynStringGenerator {
        Box::new(StringGenerator {
            char_gen,
            length_gen,
            buffer: String::with_capacity(16)
        })
    }
}

pub fn default_charset() -> Box<Generator<Output=char>> {
    Box::new(AsciiChar::new())
}

pub fn default_string_length_generator() -> Box<Generator<Output=u64>> {
    // TODO: replace the default with a range
    Box::new(::generator::constant::ConstantGenerator::new(Some(16)))
}

impl Generator for StringGenerator {
    type Output = String;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Option<&String> {
        self.buffer.clear();
        let remaining_chars = self.length_gen.gen_value(rng).cloned().unwrap_or(16);

        for _ in 0..remaining_chars {
            let next_char = self.char_gen.gen_value(rng).cloned().unwrap_or('x');
            self.buffer.push(next_char);
        }
        Some(&self.buffer)
    }
}


impl Display for StringGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "string({}, {})", self.length_gen, self.char_gen)
    }
}

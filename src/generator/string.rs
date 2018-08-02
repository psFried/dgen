use super::{DataGenRng, DynCharGenerator, DynStringGenerator, DynUnsignedIntGenerator, Generator};
use failure::Error;
use rand::distributions::Alphanumeric;
use rand::prelude::Rng;
use std::fmt::{self, Display};
use std::marker::PhantomData;
use writer::DataGenOutput;

// TODO: fill in character generators for other common ranges of unicode code points

pub trait CharGenType: Sized + 'static {
    fn get_name() -> &'static str;
    fn gen_char(rng: &mut DataGenRng) -> char;
    fn create() -> DynCharGenerator {
        let g: CharGenerator<Self> = CharGenerator::<Self>::new();
        Box::new(g)
    }
}

pub struct AsciiAlphanumeric;
impl CharGenType for AsciiAlphanumeric {
    fn get_name() -> &'static str {
        "alphanumeric"
    }
    fn gen_char(rng: &mut DataGenRng) -> char {
        rng.sample(Alphanumeric)
    }
}

pub struct UnicodeScalar;
impl CharGenType for UnicodeScalar {
    fn get_name() -> &'static str {
        "unicode_scalar"
    }
    fn gen_char(rng: &mut DataGenRng) -> char {
        rng.gen()
    }
}

pub struct UnicodeBmp;
impl CharGenType for UnicodeBmp {
    fn get_name() -> &'static str {
        "unicode_bmp"
    }
    fn gen_char(rng: &mut DataGenRng) -> char {
        ::std::char::from_u32(rng.gen_range(1u32, 65536u32)).unwrap()
    }
}

#[derive(Clone)]
pub struct CharGenerator<T: CharGenType + 'static> {
    value: char,
    _type: PhantomData<T>,
}

impl<T: CharGenType + 'static> CharGenerator<T> {
    pub fn new<E: CharGenType>() -> CharGenerator<E> {
        CharGenerator {
            value: 'x',
            _type: PhantomData,
        }
    }
}

impl<T: CharGenType + 'static> Display for CharGenerator<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", T::get_name())
    }
}

impl<T: CharGenType + 'static> Generator for CharGenerator<T> {
    type Output = char;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&char>, Error> {
        self.value = T::gen_char(rng);
        Ok(Some(&self.value))
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
        let n: CharGenerator<T> = CharGenerator {
            value: 'x',
            _type: PhantomData,
        };
        Box::new(n)
    }
}

pub struct StringGenerator {
    char_gen: Box<Generator<Output = char>>,
    length_gen: Box<Generator<Output = u64>>,
    buffer: String,
}

impl StringGenerator {
    pub fn with_length(length_gen: DynUnsignedIntGenerator) -> DynStringGenerator {
        StringGenerator::new(length_gen, AsciiAlphanumeric::create())
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

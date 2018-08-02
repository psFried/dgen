use failure::Error;
use generator::{DataGenRng, DynCharGenerator, Generator, GeneratorArg, GeneratorType};
use interpreter::{FunctionCreator, ProgramContext};
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
        // TODO: look up the actual boundaries of the BMP range so we don't have to do this loop
        loop {
            let as_u32 = rng.gen_range(1u32, 65536u32);
            if let Some(c) = ::std::char::from_u32(as_u32) {
                return c;
            }
        }
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

pub struct UnicodeScalarFun;
impl FunctionCreator for UnicodeScalarFun {
    fn get_name(&self) -> &'static str {
        UnicodeScalar::get_name()
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[], false)
    }

    fn get_description(&self) -> &'static str {
        "Generates a random Unicode scalar value. This can be any unicode code point, except for high/low surrogate code points"
    }

    fn create(
        &self,
        _args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        Ok(GeneratorArg::Char(UnicodeScalar::create()))
    }
}

pub struct UnicodeBmpFun;
impl FunctionCreator for UnicodeBmpFun {
    fn get_name(&self) -> &'static str {
        UnicodeBmp::get_name()
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[], false)
    }

    fn get_description(&self) -> &'static str {
        "Generates a random character from the unicode basic multilingual plane"
    }

    fn create(
        &self,
        _args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        Ok(GeneratorArg::Char(UnicodeBmp::create()))
    }
}

pub struct AlphaNumericChar;
impl FunctionCreator for AlphaNumericChar {
    fn get_name(&self) -> &'static str {
        "alphanumeric"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[], false)
    }

    fn get_description(&self) -> &'static str {
        "Generates a random alpha-numeric character from the ranges a-z,A-Z,0-9"
    }

    fn create(
        &self,
        _args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        Ok(GeneratorArg::Char(AsciiAlphanumeric::create()))
    }
}

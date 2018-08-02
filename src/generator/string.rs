use failure::Error;
use generator::chars::*;
use generator::{
    DataGenRng, DynCharGenerator, DynStringGenerator, DynUnsignedIntGenerator, Generator,
    GeneratorArg, GeneratorType,
};
use interpreter::{FunctionCreator, ProgramContext};
use std::fmt::{self, Display};
use writer::DataGenOutput;

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

pub struct AlphanumericString1;
impl FunctionCreator for AlphanumericString1 {
    fn get_name(&self) -> &'static str {
        "alphanumeric_string"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[GeneratorType::UnsignedInt], false)
    }

    fn get_description(&self) -> &'static str {
        "Generates a random ascii string, using the argument to determine the length"
    }

    fn create(
        &self,
        mut args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        use generator::string::StringGenerator;
        let len_gen = args.pop().unwrap().as_uint().unwrap();
        Ok(GeneratorArg::String(StringGenerator::with_length(len_gen)))
    }
}

/// 0-arg version of asciiString
pub struct AlphanumericString0;
impl FunctionCreator for AlphanumericString0 {
    fn get_name(&self) -> &'static str {
        "alphanumeric_string"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[], false)
    }

    fn get_description(&self) -> &'static str {
        "Generates a random ascii alphanumeric string with the default length of 16 characters"
    }

    fn create(
        &self,
        _args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        Ok(GeneratorArg::String(StringGenerator::new(
            default_string_length_generator(),
            AsciiAlphanumeric::create(),
        )))
    }
}

pub struct UnicodeBmpStringFun1;
impl FunctionCreator for UnicodeBmpStringFun1 {
    fn get_name(&self) -> &'static str {
        "bmp_string"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[GeneratorType::UnsignedInt], false)
    }

    fn get_description(&self) -> &'static str {
        "Generates a string of random characters from the unicode basic multilingual plane"
    }

    fn create(
        &self,
        mut args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        let len = args.pop().unwrap().as_uint().unwrap();
        Ok(GeneratorArg::String(StringGenerator::new(
            len,
            UnicodeBmp::create(),
        )))
    }
}

pub struct StringFunction;
impl FunctionCreator for StringFunction {
    fn get_name(&self) -> &'static str {
        "string"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[GeneratorType::UnsignedInt, GeneratorType::Char], false)
    }

    fn get_description(&self) -> &'static str {
        "Generates a random string using the given length and character generators"
    }

    fn create(
        &self,
        mut args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        let charset = args.pop().unwrap().as_char().unwrap();
        let lengeh = args.pop().unwrap().as_uint().unwrap();
        Ok(GeneratorArg::String(StringGenerator::new(lengeh, charset)))
    }
}

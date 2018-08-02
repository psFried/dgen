use super::FunctionCreator;
use failure::Error;
use generator::string::{AsciiAlphanumeric, UnicodeScalar, UnicodeBmp, CharGenType, default_string_length_generator, StringGenerator};
use generator::{GeneratorArg, GeneratorType};
use interpreter::resolve::ProgramContext;

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
        Ok(GeneratorArg::String(StringGenerator::new(len, UnicodeBmp::create())))
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

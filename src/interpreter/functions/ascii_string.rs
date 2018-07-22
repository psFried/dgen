use super::FunctionCreator;
use generator::{GeneratorType, GeneratorArg};
use generator::string::{default_charset, default_string_length_generator, StringGenerator};
use interpreter::resolve::ProgramContext;
use failure::Error;


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

    fn create(&self, mut args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
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

    fn create(&self, _args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        Ok(GeneratorArg::String(StringGenerator::new(default_string_length_generator(), default_charset())))
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

    fn create(&self, _args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        Ok(GeneratorArg::Char(default_charset()))
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

    fn create(&self, mut args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        let charset = args.pop().unwrap().as_char().unwrap();
        let lengeh = args.pop().unwrap().as_uint().unwrap();
        Ok(GeneratorArg::String(StringGenerator::new(lengeh, charset)))
    }
}
use super::FunctionCreator;
use generator::{GeneratorType, GeneratorArg};
use failure::Error;


pub struct RandomAsciiString1;
impl FunctionCreator for RandomAsciiString1 {
    fn get_name(&self) -> &'static str {
        "asciiString"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[GeneratorType::UnsignedInt], false)
    }
    
    fn get_description(&self) -> &'static str {
        "Generates a random ascii string, using the argument to determine the length"
    }

    fn create(&self, mut args: Vec<GeneratorArg>) -> Result<GeneratorArg, Error> {
        use generator::string::StringGenerator;
        let len_gen = args.pop().unwrap().as_uint().unwrap();
        Ok(GeneratorArg::String(StringGenerator::with_length(len_gen)))
    }
}

/// 0-arg version of asciiString
pub struct RandomAsciiString0;
impl FunctionCreator for RandomAsciiString0 {
    fn get_name(&self) -> &'static str {
        "asciiString"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[], false)
    }
    
    fn get_description(&self) -> &'static str {
        "Generates a random ascii string with the default length of 16 characters"
    }

    fn create(&self, _args: Vec<GeneratorArg>) -> Result<GeneratorArg, Error> {
        use generator::string::{default_charset, default_string_length_generator, StringGenerator};
        Ok(GeneratorArg::String(StringGenerator::new(default_string_length_generator(), default_charset())))
    }
}

pub struct AlphaNumeric;
impl FunctionCreator for AlphaNumeric {
    fn get_name(&self) -> &'static str {
        "alphanum"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[], false)
    }
    
    fn get_description(&self) -> &'static str {
        "Generates a random alpha-numeric character from the ranges a-z,A-Z,0-9"
    }

    fn create(&self, _args: Vec<GeneratorArg>) -> Result<GeneratorArg, Error> {
        Ok(GeneratorArg::Char(::generator::string::default_charset()))
    }
}
mod ascii_string;

use generator::{GeneratorType, GeneratorArg};

pub trait FunctionCreator: Send + Sync + 'static {
    fn get_name(&self) -> &'static str;
    fn get_arg_types(&self) -> (&'static [GeneratorType], bool);
    fn create(&self, args: Vec<GeneratorArg>) -> GeneratorArg;
}



pub static ALL_FUNCTIONS: &[&FunctionCreator] = &[
    &self::ascii_string::AlphaNumeric as &FunctionCreator,
    &self::ascii_string::RandomAsciiString0 as &FunctionCreator,
    &self::ascii_string::RandomAsciiString1 as &FunctionCreator
];
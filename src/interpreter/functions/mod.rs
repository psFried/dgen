mod ascii_string;
mod unsigned_int;
mod one_of;
mod either;
mod concat;

use interpreter::resolve::ProgramContext;
use generator::{GeneratorType, GeneratorArg};
use failure::Error;

pub trait FunctionCreator: 'static {
    fn get_name(&self) -> &str;
    fn get_arg_types(&self) -> (&[GeneratorType], bool);
    fn get_description(&self) -> &str;
    fn create(&self, args: Vec<GeneratorArg>, ctx: &ProgramContext) -> Result<GeneratorArg, Error>;
}

const BUILTIN_FUNCTIONS: &[&FunctionCreator] = &[
    &self::ascii_string::AlphaNumeric as &FunctionCreator,
    &self::ascii_string::RandomAsciiString0 as &FunctionCreator,
    &self::ascii_string::RandomAsciiString1 as &FunctionCreator,

    &self::unsigned_int::UnsignedInt0 as &FunctionCreator,
    &self::unsigned_int::UnsignedInt1 as &FunctionCreator,
    &self::unsigned_int::UnsignedInt2 as &FunctionCreator,

    &self::one_of::OneOfUint as &FunctionCreator,
    &self::one_of::OneOfString as &FunctionCreator,

    &self::either::EitherFun as &FunctionCreator,
    &self::either::EitherFreqFun as &FunctionCreator,
];

pub fn get_builtin_functions() -> &'static [&'static FunctionCreator] {
    BUILTIN_FUNCTIONS
}

pub struct FunctionHelp<'a>(pub &'a FunctionCreator);

use std::fmt;
impl <'a> fmt::Display for  FunctionHelp<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.0.get_name())?;

        let mut first = true;
        for arg in self.0.get_arg_types().0.iter() {
            if !first {
                f.write_str(", ")?;
            } else {
                first = false;
            }
            write!(f, "{}", arg)?;
        }

        write!(f, ") - {}", self.0.get_description())
    }
}


mod strings;
mod concat;
mod either;
mod file;
mod one_of;
mod repeat;
mod unsigned_int;
mod stable_select;

use failure::Error;
use generator::{GeneratorArg, GeneratorType};
use interpreter::resolve::ProgramContext;

pub trait FunctionCreator: 'static {
    fn get_name(&self) -> &str;
    fn get_arg_types(&self) -> (&[GeneratorType], bool);
    fn get_description(&self) -> &str;
    fn create(&self, args: Vec<GeneratorArg>, ctx: &ProgramContext) -> Result<GeneratorArg, Error>;
}

const BUILTIN_FUNCTIONS: &[&FunctionCreator] = &[
    &self::strings::AlphaNumericChar as &FunctionCreator,
    &self::strings::AlphanumericString0 as &FunctionCreator,
    &self::strings::AlphanumericString1 as &FunctionCreator,
    &self::strings::UnicodeBmpFun as &FunctionCreator,
    &self::strings::UnicodeScalarFun as &FunctionCreator,
    &self::strings::UnicodeBmpStringFun1 as &FunctionCreator,
    &self::strings::StringFunction as &FunctionCreator,
    &self::unsigned_int::UnsignedInt0 as &FunctionCreator,
    &self::unsigned_int::UnsignedInt1 as &FunctionCreator,
    &self::unsigned_int::UnsignedInt2 as &FunctionCreator,
    &self::one_of::OneOfUint as &FunctionCreator,
    &self::one_of::OneOfString as &FunctionCreator,
    &self::either::EitherFun as &FunctionCreator,
    &self::either::EitherFreqFun as &FunctionCreator,
    &self::concat::SimpleConcat as &FunctionCreator,
    &self::concat::ConcatDelimitedFun as &FunctionCreator,
    &self::repeat::RepeatFun as &FunctionCreator,
    &self::repeat::RepeatDelimitedFun as &FunctionCreator,
    &self::file::SelectFromFileFun as &FunctionCreator,
    &self::file::WordsFunction as &FunctionCreator,
    &self::stable_select::StableSelectFun as &FunctionCreator,
];

pub fn get_builtin_functions() -> &'static [&'static FunctionCreator] {
    BUILTIN_FUNCTIONS
}

pub struct FunctionHelp<'a>(pub &'a FunctionCreator);

use std::fmt;
impl<'a> fmt::Display for FunctionHelp<'a> {
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

/// returns the bottom type of the generator args. Panics if the args slice is empty
fn get_bottom_argument_type(args: &[GeneratorArg]) -> GeneratorType {
    let initial_type = args[0].get_type();
    args.iter().fold(initial_type, |target_type, arg| {
        let arg_type = arg.get_type();
        if arg_type == target_type {
            target_type
        } else {
            GeneratorType::String
        }
    })
}

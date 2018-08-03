use failure::Error;
use generator::{self, GeneratorArg, GeneratorType};
use interpreter::resolve::ProgramContext;

pub trait FunctionCreator: 'static {
    fn get_name(&self) -> &str;
    fn get_arg_types(&self) -> (&[GeneratorType], bool);
    fn get_description(&self) -> &str;
    fn create(&self, args: Vec<GeneratorArg>, ctx: &ProgramContext) -> Result<GeneratorArg, Error>;
}

const BUILTIN_FUNCTIONS: &[&FunctionCreator] = &[
    // char generators
    &generator::chars::AlphaNumericChar as &FunctionCreator,
    &generator::chars::UnicodeScalarFun as &FunctionCreator,
    &generator::chars::UnicodeBmpFun as &FunctionCreator,
    // string generators
    &generator::string::AlphanumericString0 as &FunctionCreator,
    &generator::string::AlphanumericString1 as &FunctionCreator,
    &generator::string::UnicodeBmpStringFun1 as &FunctionCreator,
    &generator::string::StringFunction as &FunctionCreator,
    // unsigned integers
    &generator::uint::UnsignedInt0 as &FunctionCreator,
    &generator::uint::UnsignedInt1 as &FunctionCreator,
    &generator::uint::UnsignedInt2 as &FunctionCreator,
    // Signed integers
    &generator::int::SignedIntFun0 as &FunctionCreator,
    &generator::int::SignedIntFun2 as &FunctionCreator,
    &generator::int::SignedIntMin as &FunctionCreator,
    &generator::int::SignedIntMax as &FunctionCreator,
    // various generators that select from among their arguments
    &generator::one_of::OneOfUint as &FunctionCreator,
    &generator::one_of::OneOfString as &FunctionCreator,
    &generator::either::EitherFun as &FunctionCreator,
    &generator::either::EitherFreqFun as &FunctionCreator,
    &generator::stable_select::StableSelectFun as &FunctionCreator,
    // generators that compose their arguments
    &generator::concat::SimpleConcat as &FunctionCreator,
    &generator::concat::ConcatDelimitedFun as &FunctionCreator,
    &generator::repeat::RepeatFun as &FunctionCreator,
    &generator::repeat::RepeatDelimitedFun as &FunctionCreator,
    // selecting from files
    &generator::file::SelectFromFileFun as &FunctionCreator,
    &generator::file::WordsFunction as &FunctionCreator,
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

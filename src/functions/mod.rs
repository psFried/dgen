mod ascii_string;

use generator::{GeneratorType, GeneratorArg};

pub trait FunctionCreator: Send + Sync + 'static {
    fn get_name(&self) -> &'static str;
    fn get_arg_types(&self) -> (&'static [GeneratorType], bool);
    fn get_description(&self) -> &'static str;
    fn create(&self, args: Vec<GeneratorArg>) -> GeneratorArg;
}



pub static ALL_FUNCTIONS: &[&FunctionCreator] = &[
    &self::ascii_string::AlphaNumeric as &FunctionCreator,
    &self::ascii_string::RandomAsciiString0 as &FunctionCreator,
    &self::ascii_string::RandomAsciiString1 as &FunctionCreator
];


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


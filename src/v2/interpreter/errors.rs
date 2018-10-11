use ::IString;
use failure::Error;
use v2::AnyFunction;
use v2::FunctionPrototype;


pub fn no_such_argument(name: IString) -> Error {
    format_err!("No such argument: '{}'", name)
}

pub fn no_such_method(name: IString, arguments: &[AnyFunction]) -> Error {
    use itertools::Itertools;
    format_err!("No such method: '{}({})'", name, arguments.iter().map(|a| a.get_type()).join(", "))
}

pub fn ambiguous_varargs_functions(name: IString, arguments: &[AnyFunction], option1: &FunctionPrototype, option2: &FunctionPrototype) -> Error {
    unimplemented!()
}
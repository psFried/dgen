use ::IString;
use failure::Error;
use v2::AnyFunction;
use v2::FunctionPrototype;
use itertools::Itertools;

pub fn no_such_argument(name: IString) -> Error {
    format_err!("No such argument: '{}'", name)
}

pub fn no_such_method(name: IString, arguments: &[AnyFunction]) -> Error {
    use itertools::Itertools;
    format_err!("No such method: '{}({})'", name, arguments.iter().map(|a| a.get_type()).join(", "))
}

pub fn ambiguous_varargs_functions(name: IString, arguments: &[AnyFunction], option1: &FunctionPrototype, option2: &FunctionPrototype) -> Error {
    let actual_arg_types = arguments.iter().map(AnyFunction::get_type).join(", ");

    format_err!("Ambiguous function call: '{}({})' could refer to two or more function prototypes:\nA: {}\nB: {}\n",
        name, actual_arg_types, option1, option2)
}
use super::find_named_functions;
use generator::GeneratorType;
use interpreter::functions::FunctionHelp;
use std::fmt::{self, Display};
use IString;

#[derive(Debug, Fail)]
pub struct ResolveError {
    message: &'static str,
    called_function: IString,
    provided_args: Vec<GeneratorType>,
}

impl ResolveError {
    pub fn no_such_function_name(name: IString, provided_args: Vec<GeneratorType>) -> ResolveError {
        ResolveError::new("no such function", name, provided_args)
    }

    pub fn mismatched_function_args(
        name: IString,
        provided_args: Vec<GeneratorType>,
    ) -> ResolveError {
        ResolveError::new("invalid function arguments", name, provided_args)
    }

    pub fn ambiguous_function_call(
        name: IString,
        provided_args: Vec<GeneratorType>,
    ) -> ResolveError {
        ResolveError::new(
            "ambiguous call to an overloaded function",
            name,
            provided_args,
        )
    }

    pub fn new(
        message: &'static str,
        called_function: IString,
        provided_args: Vec<GeneratorType>,
    ) -> ResolveError {
        ResolveError {
            message,
            called_function,
            provided_args,
        }
    }
}

impl Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Resolve Error: {}: called '{}(",
            self.message, self.called_function
        )?;
        let mut first = true;
        for arg in self.provided_args.iter() {
            if !first {
                f.write_str(", ")?;
            } else {
                first = false;
            }
            write!(f, "{}", arg)?;
        }
        f.write_str(")'")?;

        // TODO: don't lookup other possible functions from here, force them to be passed in when the error struct is initialized
        let mut first = true;
        for matching in find_named_functions(self.called_function.clone()) {
            if first {
                f.write_str("\nother possible functions are: \n")?;
                first = false;
            }
            write!(f, "{}\n", FunctionHelp(matching))?;
        }
        Ok(())
    }
}

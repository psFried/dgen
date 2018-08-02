mod ast;
pub mod functions;
mod grammar;
#[cfg(test)]
mod parse_test;
mod parser;
mod resolve;

use self::parser::{parse_library, parse_program};
use failure::Error;
use generator::{GeneratorArg, GeneratorType};

pub use self::functions::FunctionCreator;
pub use self::resolve::ProgramContext;

pub struct Interpreter {
    context: ProgramContext,
    // TODO: handle verbosity in a less hacky way
    verbosity: u64,
}

impl Interpreter {
    pub fn new(verbosity: u64) -> Interpreter {
        Interpreter {
            context: ProgramContext::new(),
            verbosity,
        }
    }

    pub fn eval_library(&mut self, lib: &str) -> Result<(), Error> {
        let ast = parse_library(lib)?;
        if self.verbosity >= 3 {
            eprintln!("LIBRARY AST: {:?}", ast);
        }
        self.context.add_lib(ast);
        Ok(())
    }

    pub fn eval_program(&mut self, program: &str) -> Result<GeneratorArg, Error> {
        let ast = parse_program(program)?;
        if self.verbosity >= 3 {
            eprintln!("PROGRAM AST: {:?}", ast);
        }
        self.context.resolve_program(ast)
    }

    pub fn function_iter(&self) -> impl Iterator<Item = &FunctionCreator> {
        self.context.function_iter()
    }
}

/// returns the bottom type of the generator args. Panics if the args slice is empty
pub fn get_bottom_argument_type(args: &[GeneratorArg]) -> GeneratorType {
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

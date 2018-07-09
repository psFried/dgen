mod ast;
mod grammar;
mod parser;
mod resolve;
pub mod functions;
#[cfg(test)] mod parse_test;

use self::parser::parse_program;
use self::resolve::ProgramContext;
use generator::GeneratorArg;
use failure::Error;


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

    pub fn eval_program(&mut self, program: &str) -> Result<GeneratorArg, Error> {
        let ast = parse_program(program)?;
        if self.verbosity >= 3 {
            eprintln!("AST: {:?}", ast);
        }
        self.context.resolve_program(&ast)
    }
}
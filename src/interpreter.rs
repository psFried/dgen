use parser::parse_program;
use resolve::ProgramContext;
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
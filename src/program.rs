use failure::Error;
use writer::DataGenOutput;

use ::interpreter::{Interpreter, UnreadSource};
use ::ProgramContext;

pub struct Program {
    iterations: u64,
    source: UnreadSource,
    rng: ProgramContext,
    interpreter: Interpreter,
}

impl Program {
    pub fn new<T: Into<UnreadSource>>(
        _verbosity: u64,
        iterations: u64,
        source: T,
        rng: ProgramContext,
    ) -> Program {
        Program {
            iterations,
            source: source.into(),
            rng,
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(self, output: &mut DataGenOutput) -> Result<(), Error> {
        let Program {
            iterations,
            source,
            mut rng,
            mut interpreter,
            ..
        } = self;

        let gen = interpreter.eval(source)?;

        for _ in 0..iterations {
            gen.write_value(&mut rng, output)?;
        }
        output.flush().map_err(Into::into)
    }

    pub fn add_std_lib(&mut self) {
        self.interpreter.add_std_lib();
    }

    pub fn add_library<T: Into<UnreadSource>>(&mut self, lib_source: T) -> Result<(), Error> {
        let source = lib_source.into();
        self.interpreter.add_module(source)
    }
}

use failure::Error;
use writer::DataGenOutput;

use v2::interpreter::{Interpreter, Source};
use v2::ProgramContext;
use libraries::STDLIBS;

pub struct Program {
    iterations: u64,
    source: Source,
    rng: ProgramContext,
    interpreter: Interpreter,
}

impl Program {
    pub fn new<T: Into<Source>>(
        verbosity: u64,
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

        let src_string = source.read_to_str()?;
        let gen = interpreter.eval(src_string.as_ref())?;

        for _ in 0..iterations {
            gen.write_value(&mut rng, output)?;
        }
        output.flush().map_err(Into::into)
    }

    pub fn add_std_lib(&mut self) {
        for lib in STDLIBS {
            self.add_library(Source::Builtin(lib)).unwrap();
        }
    }

    pub fn add_library<T: Into<Source>>(&mut self, lib_source: T) -> Result<(), Error> {
        let source = lib_source.into();
        let name = source.get_name();
        let as_str = source.read_to_str()?;
        self.interpreter.add_module(name, as_str.as_ref())
    }
}

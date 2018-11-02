mod help;

use failure::Error;
use interpreter::{Interpreter, UnreadSource};
use writer::DataGenOutput;
use ProgramContext;

pub use self::help::Help;

pub trait DgenCommand: Sized {
    fn execute(self, out: &mut DataGenOutput) -> Result<(), Error>;
}

pub struct Runner {
    iterations: u64,
    source: UnreadSource,
    runtime_context: ProgramContext,
    interpreter: Interpreter,
}

impl DgenCommand for Runner {
    fn execute(self, out: &mut DataGenOutput) -> Result<(), Error> {
        self.run(out)
    }
}

impl Runner {
    pub fn new<S: Into<UnreadSource>>(
        iterations: u64,
        source: S,
        runtime_context: ProgramContext,
        interpreter: Interpreter,
    ) -> Runner {
        Runner {
            iterations,
            source: source.into(),
            runtime_context,
            interpreter,
        }
    }

    pub fn run(self, output: &mut DataGenOutput) -> Result<(), Error> {
        let Runner {
            iterations,
            source,
            mut runtime_context,
            mut interpreter,
            ..
        } = self;

        let gen = interpreter.eval(source)?;

        for _ in 0..iterations {
            let result = gen.write_value(&mut runtime_context, output);
            if let Some(err) = result.as_ref().err() {
                handle_error(&mut runtime_context, err);
            }
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

fn handle_error(context: &mut ProgramContext, error: &Error) {
    use std::fmt::Write;

    if let Some(mut out) = context.error_output(::verbosity::VERBOSE) {
        writeln!(out, "Program Runtime Error: {}", error).expect(MUY_MALO);
    }
    if let Some(mut out) = context.error_output(::verbosity::DGEN_DEBUG) {
        writeln!(out, "{}", error.backtrace()).expect(MUY_MALO);
    }

    if let Some(program_error) = context.reset_error() {
        // program_error should not generally indicate an error/bug in dgen itself
        // it is generally caused by invalid code that was passed to the interpreter
        if let Some(mut out) = context.error_output(::verbosity::QUIET) {
            writeln!(out, "{}", program_error).expect(MUY_MALO);
        }
    }
}

const MUY_MALO: &str = "Failed to print to error stream";

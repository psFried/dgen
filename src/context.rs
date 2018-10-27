use rand::distributions::uniform::SampleUniform;
use rand::distributions::{Distribution, Standard};
use rand::prng::XorShiftRng;
use rand::{Rng, SeedableRng, FromEntropy};
use std::fmt;
use std::io;
use verbosity::Verbosity;
use interpreter::SourceRef;
use failure::Error;
use IString;

pub struct ProgramContext {
    rng: XorShiftRng,
    verbosity: Verbosity,
    is_unwinding: bool,
    error_output: Box<io::Write>,
    error: Option<ProgramRuntimeError>,
}

impl ProgramContext {
    pub fn from_seed(seed: [u8; 16], verbosity: Verbosity) -> ProgramContext {
        let rng = XorShiftRng::from_seed(seed);
        ProgramContext::new(rng, verbosity)
    }

    pub fn from_random_seed(verbosity: Verbosity) -> ProgramContext {
        let rng = XorShiftRng::from_entropy();
        ProgramContext::new(rng, verbosity)
    }

    pub fn new(rng: XorShiftRng, verbosity: Verbosity) -> ProgramContext {
        ProgramContext { 
            rng,
            verbosity,
            is_unwinding: false,
            error_output: Box::new(io::stderr()),
            error: None,
        }
    }

    pub fn error(&mut self, function_name: &IString, source_ref: &SourceRef, error: &Error) {
        if !self.is_unwinding {
            self.is_unwinding = true;
            let error = ProgramRuntimeError::new(error, self.verbosity);
            self.error = Some(error);
        }

        if self.verbosity.should_print_stacktrace() {
            self.error.as_mut().unwrap().push_stack_frame(function_name, source_ref);
        }
    }

    pub fn reset_error(&mut self) -> Option<ProgramRuntimeError> {
        self.is_unwinding = false;
        self.error.take()
    }


    #[allow(dead_code)]
    pub fn gen_value<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        self.rng.gen()
    }

    pub fn gen_range_exclusive<T: PartialOrd + Copy + SampleUniform>(
        &mut self,
        min_inclusive: T,
        max_inclusive: T,
    ) -> T {
        use rand::distributions::Uniform;
        let min = if min_inclusive < max_inclusive {
            min_inclusive
        } else {
            max_inclusive
        };
        let max = if min_inclusive > max_inclusive {
            min_inclusive
        } else {
            max_inclusive
        };
        let distribution = Uniform::new(min, max);
        distribution.sample(&mut self.rng)
    }

    pub fn gen_range_inclusive<T: PartialOrd + Copy + SampleUniform>(
        &mut self,
        min_inclusive: T,
        max_inclusive: T,
    ) -> T {
        use rand::distributions::Uniform;
        let min = if min_inclusive < max_inclusive {
            min_inclusive
        } else {
            max_inclusive
        };
        let max = if min_inclusive > max_inclusive {
            min_inclusive
        } else {
            max_inclusive
        };
        let distribution = Uniform::new_inclusive(min, max);
        distribution.sample(&mut self.rng)
    }


    pub fn error_output(&mut self, verbosity: Verbosity) -> Option<ErrorOutput> {
        if self.verbosity >= verbosity {
            Some(ErrorOutput {
                context: self
            })
        } else {
            None
        }
    }
}


pub struct ErrorOutput<'a> {
    // reference to the context to prevent this object from outliving the context
    // so that we can be gruaranteed that no RunnableFunction can output errors 
    // unless it is the currently active function
    context: &'a mut ProgramContext,
}

impl<'a> fmt::Write for ErrorOutput<'a> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.context.error_output.write_all(value.as_bytes()).map_err(|_| fmt::Error)
    }
}

#[derive(Debug)]
pub struct ProgramRuntimeError {
    verbosity: Verbosity,
    error_message: String,
    stacktrace: Vec<StackFrame>,
}

impl ProgramRuntimeError {
    fn new(error: &Error, verbosity: Verbosity) -> ProgramRuntimeError {
        let error_message = if verbosity.should_print_debug_stacktrace() {
            // if we're in debug mode then we'll render the debug version of the error
            format!("Program Runtime Error: {:#?}", error)
        } else {
            format!("Program Runtime Error: {}", error)
        };
        
        ProgramRuntimeError {
            verbosity,
            error_message,
            stacktrace: Vec::with_capacity(32),
        }
    }

    fn push_stack_frame(&mut self, function_name: &IString, source: &SourceRef) {
        self.stacktrace.push(StackFrame{ 
            function_name: function_name.clone(),
            source_ref: source.clone(),
        });
    }
}

#[derive(Debug, Clone)]
struct StackFrame {
    function_name: IString,
    source_ref: SourceRef,
}

impl fmt::Display for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} at {}:{}", self.function_name, self.source_ref.description(), self.source_ref.start_line_number())
    }
}

impl fmt::Display for ProgramRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        writeln!(f, "{}", self.error_message.as_str())?;
        if self.verbosity.is_verbose() && !self.stacktrace.is_empty() {
            // if we're being verbose, then print source_ref of the first stack frame
            let source_ref = &self.stacktrace[0].source_ref;
            writeln!(f, "{}", source_ref)?;
        }

        if self.verbosity.should_print_stacktrace() {
            writeln!(f, "Stacktrace: ")?;
            for (index, frame) in self.stacktrace.iter().enumerate() {
                writeln!(f, "{:>3}: {}", index, frame)?;
            }
        }
        Ok(())
    }
}

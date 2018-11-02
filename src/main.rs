#[macro_use]
extern crate structopt;
extern crate failure;
extern crate dgen;

mod cli_opts;

use self::cli_opts::{CliOptions, SubCommand, HelpOptions};
use dgen::interpreter::Interpreter;
use dgen::program::{DgenCommand, Help, Runner};
use dgen::repl::Repl;
use dgen::verbosity::Verbosity;
use dgen::{DataGenOutput, ProgramContext};
use failure::Error;
use std::io;
use structopt::StructOpt;

trait OrBail<T> {
    fn or_bail(self, verbosity: Verbosity) -> T;
}

impl<T> OrBail<T> for Result<T, Error> {
    fn or_bail(self, verbosity: Verbosity) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                if verbosity.should_print_error() {
                    eprintln!("Error: {}", e);
                }
                if verbosity.should_print_debug_stacktrace() {
                    eprintln!("cause: {}", e.as_fail());
                    eprintln!("backtrace: {}", e.backtrace());
                }
                ::std::process::exit(1);
            }
        }
    }
}

fn main() {
    // this call will print help and exit if --help is passed or args are invalid
    let mut args = CliOptions::from_args();
    let verbosity = args.get_verbosity();
    if verbosity.should_print_debug_stacktrace() {
        // backtraces won't get generated unless this variable is set
        std::env::set_var("RUST_BACKTRACE", "1")
    }

    let interpreter = create_interpreter(&args);

    let subcommand = args.subcommand.take();
    match subcommand {
        Some(SubCommand::Help(HelpOptions {
            function_name,
            module_name,
        })) => {
            print_function_help(function_name, module_name, interpreter, verbosity);
        }
        None => {
            let context = create_context(&args);

            if let Some(program_source) = args.get_program_source() {
                let iterations = args.iteration_count;
                let runner = Runner::new(iterations, program_source, context, interpreter);
                run_program(runner).or_bail(verbosity);
            } else {
                let repl = Repl::new(context, interpreter);
                repl.run().or_bail(verbosity);
            }
        }
    }
}

fn create_interpreter(options: &CliOptions) -> Interpreter {
    let verbosity = options.get_verbosity();
    let mut interpreter = Interpreter::new();
    if !options.no_std_lib {
        interpreter.add_std_lib();
    }
    for lib in options.get_library_sources() {
        interpreter.add_module(lib).or_bail(verbosity);
    }
    interpreter
}

fn create_context(args: &CliOptions) -> ProgramContext {
    let verbosity = args.get_verbosity();
    args.seed
        .as_ref()
        .map(|s| {
            let resolved_seed = string_to_byte_array(s);
            ProgramContext::from_seed(resolved_seed, verbosity)
        }).unwrap_or_else(|| ProgramContext::from_random_seed(verbosity))
}

fn string_to_byte_array(string: &str) -> [u8; 16] {
    let mut result = [0u8; 16];
    for (i, byte) in string.as_bytes().iter().enumerate().take(16) {
        result[i] = *byte;
    }
    result
}

fn print_function_help(
    function_name: Option<String>,
    module_name: Option<String>,
    mut interpreter: Interpreter,
    verbosity: Verbosity,
) {
    let help = Help::new(
        module_name,
        function_name,
        &mut interpreter,
        verbosity.is_verbose(),
    );
    let sout = io::stdout();
    let mut lock = sout.lock();
    let mut out = DataGenOutput::new(&mut lock);

    help.execute(&mut out).or_bail(verbosity);
}

fn run_program(program: Runner) -> Result<(), Error> {
    let sout = std::io::stdout();
    // lock stdout once at the beginning so we don't have to keep locking/unlocking it
    let mut lock = sout.lock();
    let mut output = dgen::DataGenOutput::new(&mut lock);

    program.run(&mut output)
}

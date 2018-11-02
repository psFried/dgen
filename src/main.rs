#[macro_use]
extern crate structopt;
#[macro_use]
extern crate failure;
extern crate dgen;
extern crate itertools;
extern crate lalrpop_util;
extern crate rand;
extern crate regex;
extern crate string_cache;

mod cli_opts;

use self::cli_opts::{CliOptions, SubCommand};
use dgen::interpreter::{Interpreter, UnreadSource};
use dgen::program::{DgenCommand, Help, Runner};
use dgen::verbosity::Verbosity;
use dgen::{DataGenOutput, ProgramContext};
use dgen::repl::Repl;
use failure::Error;
use std::io;
use std::path::PathBuf;
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
    let args = CliOptions::from_args();
    let verbosity = args.get_verbosity();
    if verbosity.should_print_debug_stacktrace() {
        // backtraces won't get generated unless this variable is set
        std::env::set_var("RUST_BACKTRACE", "1")
    }
    //eprintln!("sli args: {:#?}", args);
    let CliOptions {
        subcommand,
        mut files,
        ..
    } = args;
    match subcommand {
        Some(SubCommand::Help {
            function_name,
            module_name,
        }) => print_function_help(function_name, module_name, verbosity),
        Some(SubCommand::RunProgram {
            program,
            iteration_count,
            program_file,
            stdin,
            libraries,
            no_std_lib,
            seed,
        }) => {
            let source = get_program_source(program, program_file, stdin).or_bail(verbosity);
            let rng = create_rng(seed, verbosity);
            let program = create_program(
                source,
                verbosity,
                iteration_count,
                libraries,
                rng,
                !no_std_lib,
            ).or_bail(verbosity);
            run_program(program).or_bail(verbosity)
        }
        None => {
            if files.is_empty() {
                let repl = Repl::new(ProgramContext::from_random_seed(verbosity));
                repl.run().map_err(Into::into).or_bail(verbosity);
            } else {
                let program_file = files.pop().unwrap();

                let source = UnreadSource::file(program_file);
                let runtime_context = ProgramContext::from_random_seed(verbosity);
                let program = create_program(
                    source,
                    verbosity,
                    1,
                    files,
                    runtime_context,
                    true, // to add the std library. This default can only be disabled when using the `run` subcommand
                ).or_bail(verbosity);
                run_program(program).or_bail(verbosity)
            }
        }
    }
}

fn create_rng(seed: Option<String>, verbosity: Verbosity) -> ProgramContext {
    seed.map(|s| {
        let resolved_seed = string_to_byte_array(s);
        ProgramContext::from_seed(resolved_seed, verbosity)
    }).unwrap_or_else(|| ProgramContext::from_random_seed(verbosity))
}

fn string_to_byte_array(string: String) -> [u8; 16] {
    let mut result = [0u8; 16];
    for (i, byte) in string.as_bytes().iter().enumerate().take(16) {
        result[i] = *byte;
    }
    result
}

fn get_program_source(
    program_string: Option<String>,
    program_file: Option<PathBuf>,
    stdin: bool,
) -> Result<UnreadSource, Error> {
    let maybe_source = if stdin {
        Some(UnreadSource::stdin())
    } else if program_string.is_some() {
        program_string.map(Into::into)
    } else if program_file.is_some() {
        program_file.map(Into::into)
    } else {
        None
    };

    maybe_source.ok_or_else(|| format_err!("Must specify one of program, program-file, or stdin"))
}

fn create_program(
    program_source: UnreadSource,
    verbosity: Verbosity,
    iterations: u64,
    libraries: Vec<PathBuf>,
    rng: ProgramContext,
    add_std_lib: bool,
) -> Result<Runner, Error> {
    let mut program = Runner::new(verbosity, iterations, program_source, rng);

    if add_std_lib {
        program.add_std_lib();
    }
    for lib in libraries {
        program.add_library(UnreadSource::file(lib))?;
    }
    Ok(program)
}

fn print_function_help(
    function_name: Option<String>,
    module_name: Option<String>,
    verbosity: Verbosity,
) {
    let mut interpreter = Interpreter::new();
    interpreter.add_std_lib();

    let help = Help::new(
        function_name,
        module_name,
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

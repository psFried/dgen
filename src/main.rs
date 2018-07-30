#[macro_use]
extern crate structopt;
#[macro_use]
extern crate failure;
extern crate lalrpop_util;
extern crate rand;
extern crate regex;

mod cli_opts;
#[cfg(test)]
mod fun_test;
mod generator;
mod interpreter;
mod libraries;
mod writer;
mod program;

use self::program::{Program, Source};
use self::cli_opts::{CliOptions, SubCommand};
use self::interpreter::functions::{FunctionCreator, FunctionHelp};
use self::interpreter::Interpreter;
use self::generator::DataGenRng;
use rand::FromEntropy;
use failure::Error;
use std::path::PathBuf;
use structopt::StructOpt;

trait OrBail<T> {
    fn or_bail(self, verbosity: u64) -> T;
}

impl<T> OrBail<T> for Result<T, Error> {
    fn or_bail(self, verbosity: u64) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error: {}", e);
                if print_backtraces(verbosity) {
                    eprintln!("cause: {}", e.cause());
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
    let verbosity = args.debug;
    if print_backtraces(verbosity) {
        // backtraces won't get generated unless this variable is set
        std::env::set_var("RUST_BACKTRACE", "1")
    }
    match args.subcommand {
        SubCommand::ListFunctions { name } => list_functions(name),
        SubCommand::RunProgram {
            program,
            iteration_count,
            program_file,
            stdin,
            libraries,
        } => {
            let source = get_program_source(program, program_file, stdin).or_bail(verbosity);
            let rng = create_rng();
            let program = create_program(source, verbosity, iteration_count, libraries, rng, true).or_bail(verbosity);
            run_program(program).or_bail(verbosity)
        }
    }
}

// TODO: allow passing in the seed used in the rng
fn create_rng() -> DataGenRng {
    DataGenRng::from_entropy()
}

fn get_program_source(
    program_string: Option<String>,
    program_file: Option<PathBuf>,
    stdin: bool,
) -> Result<Source, Error> {
    let maybe_source = if stdin {
        Some(Source::stdin())
    } else if program_string.is_some() {
        program_string.map(Into::into)
    } else if program_file.is_some() {
        program_file.map(Into::into)
    } else {
        None
    };

    maybe_source.ok_or_else(|| {
        format_err!("Must specify one of program, program-file, or stdin")
    })
}

fn create_program(
    program_source: Source, 
    verbosity: u64, 
    iterations: u64, 
    libraries: Vec<PathBuf>,
    rng: DataGenRng,
    add_std_lib: bool) -> Result<Program, Error> {

    let mut program = Program::new(verbosity, iterations, program_source, rng);

    if add_std_lib {
        program.add_library(self::libraries::STANDARD_LIB)?;
    }
    for lib in libraries {
        program.add_library(Source::file(lib))?;
    }
    Ok(program)
}

fn print_backtraces(verbosity: u64) -> bool {
    verbosity >= 2
}

fn list_functions(name: Option<String>) {
    use regex::Regex;
    let name_filter = name.and_then(|n| {
        let trimmed = n.trim();
        Regex::new(trimmed)
            .map_err(|_err| {
                eprintln!("Cannot parse filter '{}' as a regex", trimmed);
                ()
            })
            .ok()
    });

    let mut interpreter = Interpreter::new(0);
    interpreter.eval_library(libraries::STANDARD_LIB).unwrap();

    for fun in interpreter.function_iter() {
        if name_filter
            .as_ref()
            .map(|filter| filter.is_match(fun.get_name()))
            .unwrap_or(true)
        {
            print_function_help(fun);
        }
    }
}

fn run_program(program: Program) -> Result<(), Error> {
    let sout = std::io::stdout();
    // lock stdout once at the beginning so we don't have to keep locking/unlocking it
    let mut lock = sout.lock();
    let mut output = self::writer::DataGenOutput::new(&mut lock);

    program.run(&mut output)
}

fn print_function_help(fun: &FunctionCreator) {
    let help = FunctionHelp(fun);
    println!("{}", help);
}


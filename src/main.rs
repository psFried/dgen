#[macro_use]
extern crate structopt;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate lalrpop_util;
extern crate rand;
extern crate regex;
extern crate string_cache;

mod cli_opts;
#[cfg(test)]
mod fun_test;
mod generator;
mod interpreter;
mod libraries;
mod program;
mod writer;

use self::cli_opts::{CliOptions, SubCommand};
use self::generator::DataGenRng;
use self::interpreter::functions::{FunctionCreator, FunctionHelp};
use self::interpreter::Interpreter;
use self::program::{Program, Source};
use failure::Error;
use std::path::PathBuf;
use structopt::StructOpt;

pub type IString = string_cache::DefaultAtom;

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
        SubCommand::ListFunctions { name } => list_functions(name, verbosity),
        SubCommand::RunProgram {
            program,
            iteration_count,
            program_file,
            stdin,
            libraries,
            no_std_lib,
            seed,
        } => {
            let source = get_program_source(program, program_file, stdin).or_bail(verbosity);
            let rng = create_rng(seed);
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
    }
}

fn create_rng(seed: Option<String>) -> DataGenRng {
    use rand::{FromEntropy, SeedableRng};

    seed.map(|s| {
        let resolved_seed = string_to_byte_array(s);
        DataGenRng::from_seed(resolved_seed)
    }).unwrap_or_else(|| DataGenRng::from_entropy())
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

    maybe_source.ok_or_else(|| format_err!("Must specify one of program, program-file, or stdin"))
}

fn create_program(
    program_source: Source,
    verbosity: u64,
    iterations: u64,
    libraries: Vec<PathBuf>,
    rng: DataGenRng,
    add_std_lib: bool,
) -> Result<Program, Error> {
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

fn list_functions(name: Option<String>, verbosity: u64) {
    use failure::ResultExt;
    use interpreter::functions::FunctionNameFilter;
    use regex::Regex;
    let name_filter =
        name.map(|n| {
            let trimmed = n.trim();
            let regex = Regex::new(trimmed)
                .context(format!("Cannot parse filter '{}' as a regex", trimmed))
                .map_err(|e| e.into())
                .or_bail(verbosity);
            FunctionNameFilter::Regex(regex)
        }).unwrap_or(FunctionNameFilter::All);

    let mut interpreter = Interpreter::new(0);
    interpreter.eval_library(libraries::STANDARD_LIB).unwrap();

    println!("Standard library functions:\n");

    for fun in interpreter.function_iter() {
        if name_filter.matches(fun.get_name()) {
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

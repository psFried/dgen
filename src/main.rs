#[macro_use] extern crate structopt;
#[macro_use] extern crate failure;
extern crate lalrpop_util;
extern crate rand;
extern crate regex;

mod parser;
mod cli_opts;
mod generator;
mod writer;
mod ast;
mod grammar;
mod functions;
mod resolve;
mod interpreter;
#[cfg(test)] mod parse_test; 
#[cfg(test)] mod fun_test;

use self::cli_opts::{CliOptions, SubCommand};
use self::generator::{GeneratorArg, DataGenRng};
use self::functions::{FunctionCreator, get_builtin_functions};
use self::writer::DataGenOutput;
use self::interpreter::Interpreter;
use structopt::StructOpt;
use failure::Error;
use rand::FromEntropy;


trait OrBail<T> {
    fn or_bail(self, verbosity: u64) -> T;
}

impl <T> OrBail<T> for Result<T, Error> {
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
        SubCommand::ListFunctions{name} => list_functions(name),
        SubCommand::RunProgram {program, iteration_count} => run_program(verbosity, iteration_count, program).or_bail(verbosity)
    }

}

fn print_backtraces(verbosity: u64) -> bool {
    verbosity >= 2
}

fn list_functions(name: Option<String>) {
    use regex::Regex;
    let name_filter = name.and_then(|n| {
        let trimmed = n.trim();
        Regex::new(trimmed).map_err(|_err| {
            eprintln!("Cannot parse filter '{}' as a regex", trimmed);
            ()
        }).ok()
    });


    for fun in get_builtin_functions().iter() {
        if name_filter.as_ref().map(|filter| filter.is_match(fun.get_name())).unwrap_or(true) {
            print_function_help(*fun);
        }
    }
}

fn run_program(verbosity: u64, iterations: u64, program: String) -> Result<(), Error> {
    let sout = std::io::stdout();
    // lock stdout once at the beginning so we don't have to keep locking/unlocking it
    let mut lock = sout.lock();
    let output = self::writer::DataGenOutput::new(&mut lock);

    let program = Program::with_new_rng(verbosity, iterations, program, output);
    program.run()
}


fn print_function_help(fun: &FunctionCreator) {
    let help = self::functions::FunctionHelp(fun);
    println!("{}", help);
}


pub struct Program<'a> {
    verbosity: u64,
    iterations: u64,
    source: String,
    rng: DataGenRng,
    output: DataGenOutput<'a>,
    interpreter: Interpreter,
}

impl <'a> Program<'a> {
    pub fn with_new_rng(verbosity: u64, iterations: u64, source: String, out: DataGenOutput<'a>) -> Program<'a> {
        Program::new(verbosity, iterations, source, DataGenRng::from_entropy(), out)
    }

    pub fn new(verbosity: u64, iterations: u64, source: String, rng: DataGenRng, output: DataGenOutput<'a>) -> Program<'a> {
        Program {
            verbosity,
            iterations,
            source,
            rng,
            output,
            interpreter: Interpreter::new(verbosity)
        }
    }

    pub fn run(self) -> Result<(), Error> {
        let Program {iterations, source, mut rng, mut output, mut interpreter, ..} = self;
        let mut generator = interpreter.eval_program(source.as_str())?; 

        for _ in 0..iterations {
            generator.write_value(&mut rng, &mut output)?;
        }
        Ok(())
    }
}
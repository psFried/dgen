#[macro_use]
extern crate structopt;
extern crate lalrpop_util;
extern crate rand;
extern crate regex;

mod cli_opts;
mod generator;
mod writer;
mod ast;
mod column_spec_parser;
mod functions;
mod resolve;

#[cfg(test)]
mod parse_test;

use self::cli_opts::{CliOptions, SubCommand};
use self::generator::GeneratorArg;
use self::functions::{FunctionCreator, ALL_FUNCTIONS};
use std::fmt::Display;
use structopt::StructOpt;


trait OrBail<T> {
    fn or_bail(self, &'static str) -> T;
}

impl <T, E> OrBail<T> for Result<T, E> where E: Display {
    fn or_bail(self, message: &'static str) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                println!("Error {}: {}", message, e);
                ::std::process::exit(1);
            }
        }
    }
}


fn parse_generator(verbosity: u64, program: String) -> GeneratorArg {
    let token = self::column_spec_parser::TokenParser::new().parse(program.as_str()).or_bail("Failed to parse program");
    if verbosity >= 3 {
        eprintln!("AST: {:#?}", token);
    }
    self::resolve::into_generator(&token).or_bail("Program compilation failed")
}

fn main() {
    // this call will print help and exit if --help is passed or args are invalid
    let args = CliOptions::from_args();
    let verbosity = args.debug;
    match args.subcommand {
        SubCommand::ListFunctions{name} => list_functions(name),
        SubCommand::RunProgram {program, iteration_count} => run_program(verbosity, iteration_count, program)
    }

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


    for fun in ALL_FUNCTIONS.iter() {
        if name_filter.as_ref().map(|filter| filter.is_match(fun.get_name())).unwrap_or(true) {
            print_function_help(*fun);
        }
    }
}

fn run_program(verbosity: u64, iterations: u64, program: String) {
    use rand::FromEntropy;
    use generator::DataGenRng;

    let mut generator = parse_generator(verbosity, program);
    let mut rng: DataGenRng = DataGenRng::from_entropy();

    let sout = std::io::stdout();
    // lock stdout once at the beginning so we don't have to keep locking/unlocking it
    let mut lock = sout.lock();
    let mut output = self::writer::DataGenOutput::new(&mut lock);

    for _ in 0..iterations {
        generator.write_value(&mut rng, &mut output).or_bail("runtime IO error");
    }
}

fn print_function_help(fun: &FunctionCreator) {
    let help = self::functions::FunctionHelp(fun);
    println!("{}", help);
}
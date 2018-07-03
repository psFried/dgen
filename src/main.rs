#[macro_use]
extern crate structopt;

#[macro_use] extern crate lalrpop_util;

//#[macro_use]
//extern crate nom;
extern crate rand;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate regex;

mod cli_opts;
mod generator;
mod formatter;
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


trait OrBail<T> {
    fn or_bail(self) -> T;
}

impl <T, E> OrBail<T> for Result<T, E> where E: Display {
    fn or_bail(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                println!("Error: {}", e);
                ::std::process::exit(1);
            }
        }
    }
}


fn parse_generator(verbosity: u64, program: String) -> GeneratorArg {
    let token = self::column_spec_parser::TokenParser::new().parse(program.as_str()).or_bail();
    if verbosity >= 3 {
        eprintln!("AST: {:#?}", token);
    }
    self::resolve::into_generator(token).or_bail()
}

fn main() {
    use structopt::StructOpt;

    env_logger::init();

    let args = self::cli_opts::CliOptions::from_args();
    let verbosity = args.debug;
    match args.subcommand {
        SubCommand::ListFunctions{name} => list_functions(verbosity, name),
        SubCommand::RunProgram {program, iteration_count} => run_program(verbosity, iteration_count, program)
    }

}

fn list_functions(verbosity: u64, name: Option<String>) {
    use regex::Regex;
    let name_filter = name.and_then(|n| {
        let trimmed = n.trim();
        Regex::new(trimmed).map_err(|err| {
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
    use rand::{Rng, SeedableRng, FromEntropy};
    use generator::DataGenRng;

    let mut generator = parse_generator(verbosity, program);
    let mut rng: DataGenRng = DataGenRng::from_entropy();

    for _ in 0..iterations {
        let result = generator.gen_displayable(&mut rng);
        if let Some(displayable) = result {
            println!("{}", displayable);
        } else {
            break;
        }
    }

}


fn print_function_help(fun: &FunctionCreator) {
    let help = self::functions::FunctionHelp(fun);
    println!("{}", help);
}
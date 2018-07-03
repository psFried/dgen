#[macro_use]
extern crate structopt;

#[macro_use] extern crate lalrpop_util;

//#[macro_use]
//extern crate nom;
extern crate rand;

#[macro_use]
extern crate log;
extern crate env_logger;

mod cli_opts;
mod generator;
mod formatter;
mod ast;
mod column_spec_parser;
mod functions;
mod resolve;

#[cfg(test)]
mod parse_test;

use self::cli_opts::CliOptions;
use self::generator::GeneratorArg;
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


fn parse_generator(args: &CliOptions) -> GeneratorArg {
    let token = self::column_spec_parser::TokenParser::new().parse(args.program.as_str()).or_bail();
    if args.debug >= 3 {
        eprintln!("AST: {:?}", token);
    }
    self::resolve::into_generator(token).or_bail()
}

fn main() {
    use structopt::StructOpt;
    use rand::{Rng, SeedableRng, FromEntropy};
    use generator::DataGenRng;

    env_logger::init();

    let args = self::cli_opts::CliOptions::from_args();

    let mut generator = parse_generator(&args);
    let mut rng: DataGenRng = DataGenRng::from_entropy();

    for _ in 0..args.iteration_count {
        let result = generator.gen_displayable(&mut rng);
        if let Some(displayable) = result {
            println!("{}", displayable);
        } else {
            break;
        }
    }

}

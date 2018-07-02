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

#[cfg(test)]
mod parse_test;


fn main() {
    env_logger::init();

    println!("Hello, world!");
}

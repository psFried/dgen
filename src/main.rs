#[macro_use]
extern crate structopt;
extern crate rand;
extern crate combine;
#[macro_use]
extern crate log;
extern crate env_logger;

mod cli_opts;
mod generator;
mod formatter;


fn main() {
    env_logger::init();

    println!("Hello, world!");
}

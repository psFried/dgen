use interpreter::ast::{Program, MacroDef};
use interpreter::grammar::{ProgramParser, LibraryParser};
use failure::Error;


pub fn parse_program(input: &str) -> Result<Program, Error> {
    ProgramParser::new().parse(input).map_err(|err| {
        format_err!("Parse Error: {}", err)
    })
}

pub fn parse_library(input: &str) -> Result<Vec<MacroDef>, Error> {
    LibraryParser::new().parse(input).map_err(|err| {
        format_err!("Parse Error: {}", err)
    })
}
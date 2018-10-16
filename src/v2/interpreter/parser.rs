use failure::Error;
use v2::interpreter::ast::{MacroDef, Program};
use v2::interpreter::grammar::{LibraryParser, ProgramParser};

pub fn parse_program(input: &str) -> Result<Program, Error> {
        ProgramParser::new().parse(input).map_err(|err| {
                format_err!("Parse Error: {}, \n Error parsing input: '{}'", err, input)
        })
}

pub fn parse_library(input: &str) -> Result<Vec<MacroDef>, Error> {
        LibraryParser::new().parse(input).map_err(|err| {
                format_err!("Parse Error: {}, \n Error parsing input: '{}'", err, input)
        })
}

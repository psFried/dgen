use ast::{Program, Expr};
use grammar::{ExprParser, ProgramParser};
use failure::Error;


pub fn parse_token(input: &str) -> Result<Expr, Error> {
    ExprParser::new().parse(input).map_err(|err| {
        format_err!("Parse Error: {}", err)
    })
}

pub fn parse_program(input: &str) -> Result<Program, Error> {
    ProgramParser::new().parse(input).map_err(|err| {
        format_err!("Parse Error: {}", err)
    })
}
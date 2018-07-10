use interpreter::ast::Program;
use interpreter::grammar::ProgramParser;
use failure::Error;


pub fn parse_program(input: &str) -> Result<Program, Error> {
    ProgramParser::new().parse(input).map_err(|err| {
        format_err!("Parse Error: {}", err)
    })
}
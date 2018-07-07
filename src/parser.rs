use ast::Token;
use column_spec_parser::TokenParser;
use failure::Error;

#[derive(Debug, Fail)]
#[fail(display = "Parse Error: {}", _0)]
pub struct ParseError(String);


pub fn parse_token(input: &str) -> Result<Token, Error> {
    TokenParser::new().parse(input).map_err(|err| {
        format_err!("Parse Error: {}", err)
    })
}
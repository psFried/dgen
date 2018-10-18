use failure::Error;
use lalrpop_util::ParseError;
use v2::interpreter::ast::{MacroDef, Program};
use v2::interpreter::grammar::{LibraryParser, ProgramParser};
use v2::interpreter::errors::SourceErrRegion;
use std::fmt::{self, Display};
use IString;


#[derive(Fail, Debug)]
pub struct PgenParseError {
    source_name: IString,
    input: String,
    inner: ParseErrorInner,
}

#[derive(Debug)]
struct ParseErrorInner {
    location: Option<usize>,
    description: String
}

impl<'a, T: Display> From<ParseError<usize, T, &'a str>> for ParseErrorInner {
    fn from(err: ParseError<usize, T, &'a str>) -> ParseErrorInner {
        let description = format!("Parse Error: {}", err);
        let location = match err {
            ParseError::InvalidToken{location} => Some(location),
            ParseError::ExtraToken { token } => Some(token.0),
            ParseError::UnrecognizedToken {token, ..} => token.map(|t| t.0),
            _ => None
        };
        ParseErrorInner {
            location,
            description
        }
    }
}

pub fn parse_program(source_name: IString, input: &str) -> Result<Program, Error> {
    ProgramParser::new().parse(input).map_err(ParseErrorInner::from).map_err(|e| {
        PgenParseError {
            source_name,
            input: input.to_owned(),
            inner: e,
        }
    }).map_err(Into::into)
}

pub fn parse_library(source_name: IString, input: &str) -> Result<Vec<MacroDef>, Error> {
    LibraryParser::new().parse(input).map_err(ParseErrorInner::from).map_err(|e| {
        PgenParseError {
            source_name,
            input: input.to_owned(),
            inner: e,
        }
    }).map_err(Into::into)
}

impl Display for PgenParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error parsing '{}': {}\n", self.source_name, self.inner.description)?;
        if let Some(offset) = self.inner.location {
            let err_region = SourceErrRegion::new(self.input.as_str(), offset);
            write!(f, "{}", err_region)?;
        }
        Ok(())
    }
}

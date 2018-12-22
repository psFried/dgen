use failure::Error;
use crate::interpreter::ast::Program;
use crate::interpreter::errors::SourceErrRegion;
use crate::interpreter::grammar::ProgramParser;
use lalrpop_util::ParseError;
use std::fmt::{self, Display};
use crate::IString;

#[derive(Fail, Debug)]
pub struct DgenParseError {
    source_name: IString,
    input: String,
    inner: ParseErrorInner,
}

impl DgenParseError {
    pub fn is_unexpected_eof(&self) -> bool {
        self.inner.is_unexpected_eof
    }
}

#[derive(Debug)]
struct ParseErrorInner {
    location: Option<usize>,
    description: String,
    is_unexpected_eof: bool,
}

impl<'a, T: Display> From<ParseError<usize, T, &'a str>> for ParseErrorInner {
    fn from(err: ParseError<usize, T, &'a str>) -> ParseErrorInner {
        let description = format!("Parse Error: {}", err);
        let location = match &err {
            ParseError::InvalidToken { location } => Some(*location),
            ParseError::ExtraToken { token } => Some(token.0),
            ParseError::UnrecognizedToken { token, .. } => token.as_ref().map(|t| t.0),
            _ => None,
        };

        let is_eof = match &err {
            ParseError::UnrecognizedToken { token, .. } => token.is_none(),
            _ => false,
        };
        ParseErrorInner {
            location,
            description,
            is_unexpected_eof: is_eof,
        }
    }
}

pub fn parse_program(source_name: IString, input: &str) -> Result<Program, Error> {
    ProgramParser::new()
        .parse(input)
        .map_err(ParseErrorInner::from)
        .map_err(|e| DgenParseError {
            source_name,
            input: input.to_owned(),
            inner: e,
        }).map_err(Into::into)
}

impl Display for DgenParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error parsing '{}': {}\n",
            self.source_name, self.inner.description
        )?;
        if let Some(offset) = self.inner.location {
            let err_region = SourceErrRegion::new(self.input.as_str(), offset);
            write!(f, "{}", err_region)?;
        }
        Ok(())
    }
}

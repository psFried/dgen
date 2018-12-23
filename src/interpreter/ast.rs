use std::fmt::{self, Display};
use std::str::Chars;
use crate::IString;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum GenType {
    String,
    Uint,
    Int,
    Decimal,
    Boolean,
    Bin,
}

impl GenType {
    pub fn display_name(&self) -> &'static str {
        match *self {
            GenType::String => "String",
            GenType::Uint => "Uint",
            GenType::Int => "Int",
            GenType::Decimal => "Decimal",
            GenType::Boolean => "Boolean",
            GenType::Bin => "Bin",
        }
    }
}

impl Display for GenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub function_name: IString,
    pub args: Vec<WithSpan<Expr>>,
    pub mapper: Option<Box<FunctionMapper>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionMapper {
    pub arg_name: IString,
    pub mapper_body: WithSpan<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    ArgumentUsage(IString),
    Function(FunctionCall),
    StringLiteral(IString),
    IntLiteral(u64),
    SignedIntLiteral(i64),
    DecimalLiteral(f64),
    BooleanLiteral(bool),
    BinaryLiteral(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WithSpan<T> {
    pub span: Span,
    pub value: T,
}

impl<T> ::std::ops::Deref for WithSpan<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroArgument {
    pub name: IString,
    pub arg_type: GenType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroDef {
    pub doc_comments: String,
    pub name: IString,
    pub args: Vec<MacroArgument>,
    pub body: WithSpan<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub assignments: Vec<WithSpan<MacroDef>>,
    pub expr: Option<WithSpan<Expr>>,
}

pub fn process_string_escapes(input: &str) -> Result<IString, &'static str> {
    let mut result = String::with_capacity(input.len());

    let mut char_iter = input.chars();
    loop {
        let next_char = {
            let c = char_iter.next();
            if c.is_none() {
                break;
            }
            c.unwrap()
        };

        if next_char == '\\' {
            // process escape sequences
            let escape_id = char_iter.next().ok_or("Unfinished escape sequence")?;
            let result_char = match escape_id {
                '\\' => '\\',
                '"' => '"',
                't' => '\t',
                'n' => '\n',
                'r' => '\r',
                'u' => process_unicode_escape(&mut char_iter)?,
                'U' => process_unicode_escape(&mut char_iter)?,
                other @ _ => {
                    eprintln!(
                        "Error in string literal, invalid escape sequence '\\{}'",
                        other
                    );
                    return Err("invalid escape sequence");
                }
            };
            result.push(result_char);
        } else {
            result.push(next_char);
        }
    }
    Ok(result.into())
}

fn process_unicode_escape(char_iter: &mut Chars) -> Result<char, &'static str> {
    const ERR_MSG: &str = "invalid unicode escape sequence";
    let l_curly = char_iter.next().ok_or(ERR_MSG)?;
    if l_curly != '{' {
        return Err(ERR_MSG);
    }

    let mut sequence = String::with_capacity(6);
    loop {
        let c = char_iter.next().ok_or(ERR_MSG)?;
        if c == '}' {
            break;
        } else {
            sequence.push(c);
        }
        if sequence.len() > 6 {
            return Err(ERR_MSG);
        }
    }

    let as_uint = u32::from_str_radix(sequence.as_str(), 16).map_err(|e| {
        eprintln!("{}: {}", ERR_MSG, e);
        ERR_MSG
    })?;

    ::std::char::from_u32(as_uint).ok_or_else(|| {
        eprintln!(
            "{}: the value '{}' is not a valid unicode codepoint",
            ERR_MSG, sequence
        );
        ERR_MSG
    })
}

pub fn process_doc_comments(raw_lines: Vec<String>) -> String {
    if raw_lines.is_empty() {
        "user defined function".to_owned()
    } else {
        raw_lines.join("\n")
    }
}
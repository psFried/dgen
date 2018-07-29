use generator::GeneratorType;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub function_name: String,
    pub args: Vec<Expr>,
    pub mapper: Option<Box<FunctionMapper>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionMapper {
    pub arg_name: String,
    pub mapper_body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Function(FunctionCall),
    StringLiteral(String),
    IntLiteral(u64),
    SignedIntLiteral(i64),
    DecimalLiteral(f64),
    BooleanLiteral(bool),
    CharLiteral(char),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnSpec {
    pub column_name: String,
    pub spec: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroArgument {
    pub name: String,
    pub arg_type: GeneratorType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroDef {
    pub doc_comments: Vec<String>,
    pub name: String,
    pub args: Vec<MacroArgument>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub assignments: Vec<MacroDef>,
    pub expr: Expr,
}

pub fn process_string_escapes(input: &str) -> Result<String, &'static str> {
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
    Ok(result)
}

fn process_unicode_escape(char_iter: &mut Chars) -> Result<char, &'static str> {
    const ERR_MSG: &'static str = "invalid unicode escape sequence";
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

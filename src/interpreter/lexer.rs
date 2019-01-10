use std::str::CharIndices;

type Loc = usize;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Delimiter {
    Paren,
    CurlyBrace,
    SquareBracket,
}

// TODO: start adding tests for the basic few things we have
// TODO: implement lexing boolean and string literals, and def keyword

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Token<'a> {
    NumberLiteral(&'a str),
    BooleanLiteral(&'a str),
    StringDelimiter(&'a str),
    StringLiteralBytes(&'a str),
    Comment(&'a str),

    OpenDelimiter(Delimiter),
    CloseDelimiter(Delimiter),

    RightSkinnyArrow,
    Semicolon,
    Comma,
    Period,

    UnaryMinus,
    UnaryPlus,

    Ident(&'a str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexicalError {
    UnexpectedEof,
    UnexpectedCharacter(char),
}


pub struct CharIter<'a> {
    inner: CharIndices<'a>,

    position: Loc,
    next: Option<(Loc, char)>,
}

impl CharIter<'_> {
    fn new(input: &str) -> CharIter {
        let mut inner = input.char_indices();
        let next = inner.next();

        CharIter {
            inner, 
            next,
            position: 0,
        }
    }


    fn peek(&self) -> Option<(Loc, char)> {
        self.next.clone()
    }

}

impl <'a> Iterator for CharIter<'a> {
    type Item = (Loc, char);

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.next.take();
        if let Some((i, _)) = result.as_ref() {
            self.position = *i;
        }
        self.next = self.inner.next();
        result
    }
}


pub struct Lexer<'a> {
    input: &'a str,
    char_iter: CharIter<'a>,
    is_newline: bool,
}


impl <'a> Lexer<'a> {

    pub fn new(input: &str) -> Lexer {
        Lexer {
            input,
            char_iter: CharIter::new(input),
            is_newline: false,
        }
    }


    fn skip_whitespace(&mut self) {
        loop {
            if let Some((_, c)) = self.char_iter.peek() {
                if c.is_whitespace() {
                    if c == '\n' {
                        self.is_newline = true;
                    }
                    let _ = self.char_iter.next();
                    continue
                }
            }
            break
        }
    }

    fn slice(&self, start: Loc, end: Loc) -> &'a str {
        &self.input[start..end]
    }

    fn take_while<F>(&mut self, mut predicate: F) -> (Loc, &'a str, Loc) where F: FnMut(char) -> bool {
        let start = self.char_iter.position;
        let mut end = start;
        while let Some((loc, next_char)) = self.char_iter.peek() {
            if predicate(next_char) {
                let _ = self.char_iter.next();
                end = loc;
            } else {
                break;
            }
        }

        (start, self.slice(start, end), end)
    }

    // fn take_at_least_one_while<F>(&mut self, start: Loc, predicate: F) -> (&'a str, Loc) where F: FnMut(char) -> bool {
    //     let result = se
    // }

    fn take_comment(&mut self) -> (Loc, Token<'a>, Loc) {
        let (start, value, end) = self.take_while(|c| { c != '\n' });
        (start, Token::Comment(value), end)
    }

    fn take_single_char(&mut self, token: Token<'a>) -> Option<Result<(Loc, Token<'a>, Loc), LexicalError>> {
        self.char_iter.next().map(|(location, _)| {
            Ok((location, token, location))
        })
    }

    fn next_token(&mut self) -> Option<Result<(Loc, Token<'a>, Loc), LexicalError>> {
        self.skip_whitespace();
        let is_first_non_whitespace_on_line = std::mem::replace(&mut self.is_newline, false);

        if let Some((current_position, current_char)) = self.char_iter.peek() {
            match current_char {
                '(' => self.take_single_char(Token::OpenDelimiter(Delimiter::Paren)),
                ')' => self.take_single_char(Token::CloseDelimiter(Delimiter::Paren)),
                '{' => self.take_single_char(Token::OpenDelimiter(Delimiter::CurlyBrace)),
                '}' => self.take_single_char(Token::CloseDelimiter(Delimiter::CurlyBrace)),
                '[' => self.take_single_char(Token::OpenDelimiter(Delimiter::SquareBracket)),
                ']' => self.take_single_char(Token::CloseDelimiter(Delimiter::SquareBracket)),


                '.' => self.take_single_char(Token::Period),
                ',' => self.take_single_char(Token::Comma),
                ';' => self.take_single_char(Token::Semicolon),

                '#' => {
                    let comment = self.take_comment();
                    if is_first_non_whitespace_on_line {
                        Some(Ok(comment))
                    } else {
                        // if there was code to the left of the comment, then we'll just skip past it and get the next token
                        self.next_token()
                    }
                }

                '+' => self.take_single_char(Token::UnaryPlus),
                '-' => {
                    self.advance();
                    match self.char_iter.peek() {
                        Some((end, '>')) => {
                            let _ = self.char_iter.next();
                            Some(Ok((current_position, Token::RightSkinnyArrow, end)))
                        }
                        _ => Some(Ok((current_position, Token::UnaryMinus, current_position)))
                    }
                }

                '0'..='9' => self.take_number_literal(),

                istart if is_valid_identifier_start(istart) => self.take_identifier(),

                other @ _ => Some(Err(LexicalError::UnexpectedCharacter(other)))
            }

        } else {
            None
        }
    }

    fn take_identifier(&mut self) -> Option<Result<(Loc, Token<'a>, Loc), LexicalError>> {
        let (start, value, end) = self.take_while(is_valid_identifier_char);
        Some(Ok((start, Token::Ident(value), end)))
    }

    fn advance(&mut self) {
        let _ = self.char_iter.next();
    }

    fn take_number_literal(&mut self) -> Option<Result<(Loc, Token<'a>, Loc), LexicalError>> {
        let (start, value, end) = self.take_while(|c| {
            c.is_digit(10)
        });

        Some(Ok((start, Token::NumberLiteral(value), end)))
    }

}


impl <'a> Iterator for Lexer<'a> {
    type Item = Result<(Loc, Token<'a>, Loc), LexicalError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn is_valid_identifier_start(start: char) -> bool {
    start.is_ascii_alphabetic() || start == '_'
}

fn is_valid_identifier_char(c: char) -> bool {
    !c.is_whitespace()
}
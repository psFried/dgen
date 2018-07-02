use column_spec_parser::{TokenParser, ColumnSpecParser};

use ast::{Token, FunctionCall};


#[test]
fn parses_int_literal_token() {
    let result = TokenParser::new().parse(r#"1234"#);
    assert_eq!(Ok(int(1234)), result);
}

#[test]
fn parses_string_literal_with_escaped_quotes() {
    let result = TokenParser::new().parse(r#""some\"str""#);
    assert_eq!(Ok(string(r#"some"str"#)), result);
}

#[test]
fn parses_string_literal_token() {
    let result = TokenParser::new().parse(r#""somestr""#);
    assert_eq!(Ok(string("somestr")), result);
}

#[test]
fn parses_decimal_literal_token() {
    let result = TokenParser::new().parse(r#"123.45"#);
    assert_eq!(Ok(float(123.45)), result);
}


#[test]
fn parses_function_call_with_literal_arguments() {
    let result = TokenParser::new().parse(r#"fun_name("foo", 55, 12.5)"#);
    let expected = FunctionCall {
        function_name: "fun_name".to_owned(),
        args: vec![
            Token::StringLiteral("foo".to_owned()),
            Token::IntLiteral(55),
            Token::DecimalLiteral(12.5)
        ]
    };
    assert_eq!(Ok(Token::Function(expected)), result);
}

#[test]
fn parses_nested_function_calls() {
    let result = TokenParser::new().parse(r#"fun1("foo", fun2(12.5, fun3(111)), "bar")"#);
    let expected = fun("fun1", vec![
        string("foo"),
        fun("fun2", vec![
            float(12.5),
            fun("fun3", vec![
                int(111)
            ])
        ]),
        string("bar")
    ]);
    assert_eq!(Ok(expected), result);
}


fn fun(name: &str, args: Vec<Token>) -> Token {
    Token::Function(FunctionCall {
        function_name: name.to_owned(),
        args
    })
}

fn string(s: &str) -> Token {
    Token::StringLiteral(s.to_owned())
}

fn int(i: i64) -> Token {
    Token::IntLiteral(i)
}

fn float(f: f64) -> Token {
    Token::DecimalLiteral(f)
}

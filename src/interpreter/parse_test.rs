use ::interpreter::ast::{
    Expr, FunctionCall, FunctionMapper, GenType, MacroArgument, MacroDef, Program, WithSpan, Span,
};
use ::interpreter::grammar::ExprParser;
use ::interpreter::parser::parse_program;
use IString;

#[test]
fn parses_boolean_literal_false_token() {
    let result = ExprParser::new().parse(r#"false"#);
    assert_eq!(Ok(with_span(0, 5, boolean(false))), result);
}

#[test]
fn parses_boolean_literal_true_token() {
    let result = ExprParser::new().parse(r#"true"#);
    assert_eq!(Ok(with_span(0, 4, boolean(true))), result);
}

#[test]
fn parses_unsigned_int_literal_token() {
    let result = ExprParser::new().parse(r#"1234"#);
    assert_eq!(Ok(with_span(0, 4, int(1234))), result);
}

#[test]
fn parses_unsigned_int_hex_literal_token() {
    let result = ExprParser::new().parse("0xFF");
    assert_eq!(Ok(with_span(0, 4, int(255))), result);
}

#[test]
fn parses_signed_int_literal_negative_token() {
    let result = ExprParser::new().parse(r#"-1234"#);
    assert_eq!(Ok(with_span(0, 5, sint(-1234))), result);
}

#[test]
fn parses_signed_int_literal_positive_token() {
    let result = ExprParser::new().parse(r#"+1234"#);
    assert_eq!(Ok(with_span(0, 5, sint(1234))), result);
}

#[test]
fn parses_string_literal_with_escaped_quotes() {
    let result = ExprParser::new().parse(r#""some\"str""#);
    assert_eq!(Ok(with_span(0, 11, string(r#"some"str"#))), result);
}

#[test]
fn parses_decimal_literal_token() {
    let result = ExprParser::new().parse(r#"123.45"#);
    assert_eq!(Ok(with_span(0, 6, float(123.45))), result);
}

#[test]
fn parses_decimal_literal_token_with_negative_sign() {
    let result = ExprParser::new().parse(r#"-123.45"#);
    assert_eq!(Ok(with_span(0, 7, float(-123.45))), result);
}

#[test]
fn parses_decimal_literal_token_with_positive_sign() {
    let result = ExprParser::new().parse(r#"+123.45"#);
    assert_eq!(Ok(with_span(0, 7, float(123.45))), result);
}

#[test]
fn parses_basic_string_literal_token() {
    string_literal_test(r#""somestr""#, "somestr");
}

#[test]
fn parses_string_literal_with_whitespace_chars_and_escape_sequences() {
    string_literal_test(r#"" some\t str\n ""#, " some\t str\n ");
}

#[test]
fn parses_string_literal_with_unicode_escape_sequences() {
    string_literal_test(r#""foo\U{1F4A9}""#, "foo💩");
}

#[test]
fn parses_string_literal_that_is_all_whitespace() {
    string_literal_test(r#"" \t \n \r ""#, " \t \n \r ");
}

#[test]
fn parses_basic_char_literal() {
    char_literal_test("'a'", 'a');
}

#[test]
fn parses_unicode_char_literal() {
    char_literal_test(r#"'\u{1f4a9}'"#, '\u{1f4a9}');
}

#[test]
fn parses_newline_escaped_char_literal() {
    char_literal_test(r#"'\n'"#, '\n');
}

#[test]
fn parses_function_call_with_literal_arguments() {
    let result = ExprParser::new().parse(r#"fun_name("foo", 55, 12.5)"#);
    let expected = FunctionCall {
        function_name: "fun_name".into(),
        args: vec![
            with_span(9, 14, Expr::StringLiteral("foo".into())),
            with_span(16, 18, Expr::IntLiteral(55)),
            with_span(20, 24, Expr::DecimalLiteral(12.5))
        ],
        mapper: None,
    };
    assert_eq!(Ok(with_span(0, 25, Expr::Function(expected))), result);
}

#[test]
fn parses_function_call_with_zero_arguments() {
    let result = ExprParser::new().parse("fun_name()");
    let expected = FunctionCall {
        function_name: "fun_name".into(),
        args: Vec::new(),
        mapper: None,
    };
    assert_eq!(Ok(with_span(0, 10, Expr::Function(expected))), result);
}

#[test]
fn parses_nested_function_calls() {
    let result = ExprParser::new().parse(r#"fun1("foo", fun2(12.5, fun3(111)), "bar")"#);
    let expected = with_span(0, 41, fun(
        "fun1",
        vec![
            with_span(5, 10, string("foo")),
            with_span(12, 33, fun("fun2", vec![
                with_span(17, 21, float(12.5)), 
                with_span(23, 32, fun("fun3", vec![
                    with_span(28, 31, int(111))
                ]))
            ])),
            with_span(35, 40, string("bar")),
        ],
    ));
    assert_eq!(Ok(expected), result);
}

#[test]
fn parses_mapped_function_call() {
    let input = r#"fun1("foo", 7) {mapper_arg ->
        inner(mapper_arg, mapper_arg)
    }"#;
    let result = ExprParser::new().parse(input);
    let expected = with_span(0, 73, mfun(
        "fun1",
        vec![with_span(5, 10, string("foo")), with_span(12, 13, int(7))],
        "mapper_arg",
        with_span(38, 67, fun(
            "inner",
            vec![with_span(44, 54, arg_usage("mapper_arg")), with_span(56, 66, arg_usage("mapper_arg"))],
        )),
    ));
    assert_eq!(Ok(expected), result);
}

#[test]
fn parses_mapped_function_call_withou_args() {
    let input = r#"fun1() {mapper_arg ->
        inner(mapper_arg, mapper_arg)
    }"#;
    let result = ExprParser::new().parse(input);
    let expected = with_span(0, 65, mfun(
        "fun1",
        Vec::new(),
        "mapper_arg",
        with_span(30, 59, fun(
            "inner",
            vec![with_span(36, 46, arg_usage("mapper_arg")), with_span(48, 58, arg_usage("mapper_arg"))],
        )),
    ));
    assert_eq!(Ok(expected), result);
}

#[test]
fn parses_program_with_macro_definitions() {
    let input = r#"
    #   comment 1    

    def wtf(count: Uint) = asciiString(count);

    # comment 2    
       #comment 3
    def foo() = wtf(uint(0, 9)); # ignored comment

# ignored comment
    foo()
    "#;
    let expected = Program {
        assignments: vec![
            with_span(28, 70, MacroDef {
                name: s("wtf"),
                args: vec![MacroArgument {
                    name: s("count"),
                    arg_type: GenType::Uint,
                }],
                body: with_span(51, 69, Expr::Function(FunctionCall {
                    function_name: s("asciiString"),
                    args: vec![with_span(63, 68, arg_usage("count"))],
                    mapper: None,
                })),
                doc_comments: "comment 1".to_owned(),
            }),
            with_span(114, 142, MacroDef {
                name: s("foo"),
                args: Vec::new(),
                body: with_span(126, 141, Expr::Function(FunctionCall {
                    function_name: s("wtf"),
                    args: vec![with_span(130, 140, Expr::Function(FunctionCall {
                        function_name: s("uint"),
                        args: vec![with_span(135, 136, Expr::IntLiteral(0)), with_span(138, 139, Expr::IntLiteral(9))],
                        mapper: None,
                    }))],
                    mapper: None,
                })),
                doc_comments: "comment 2\ncomment 3".to_owned(),
            }),
        ],
        expr: Some(with_span(184, 189, Expr::Function(FunctionCall {
            function_name: s("foo"),
            args: Vec::new(),
            mapper: None,
        }))),
    };

    let actual = parse_program("test input".into(), input).expect("failed to parse input");
    assert_eq!(expected, actual);
}

#[test]
fn parses_bin_literal() {
    let input = "[ 0x00,0xff, 0x01]";
    let expected_output = with_span(0, 18, bin(&[0x00, 0xff, 0x01]));
    let actual = ExprParser::new().parse(input).expect("failed to parse");
    assert_eq!(expected_output, actual);
}

#[test]
fn parses_empty_bin_literal() {
    let input = "[ ]";
    let expected_output = with_span(0, 3, bin(&[]));
    let actual = ExprParser::new().parse(input).expect("failed to parse");
    assert_eq!(expected_output, actual);
}


fn s(val: &str) -> IString {
    val.into()
}

fn arg_usage(name: &str) -> Expr {
    Expr::ArgumentUsage(name.into())
}

fn string_literal_test(to_parse: &str, expected: &str) {
    let actual = ExprParser::new().parse(to_parse).expect("failed to parse");
    assert_eq!(Expr::StringLiteral(s(expected)), actual.value);
}

fn char_literal_test(to_parse: &str, expected: char) {
    let actual = ExprParser::new().parse(to_parse).expect("failed to parse");
    assert_eq!(ch(expected), actual.value);
}

fn fun(name: &str, args: Vec<WithSpan<Expr>>) -> Expr {
    Expr::Function(FunctionCall {
        function_name: name.into(),
        args,
        mapper: None,
    })
}

fn mfun(name: &str, args: Vec<WithSpan<Expr>>, mapper_arg_name: &str, mapper_body: WithSpan<Expr>) -> Expr {
    Expr::Function(FunctionCall {
        function_name: name.into(),
        args,
        mapper: Some(Box::new(FunctionMapper {
            arg_name: mapper_arg_name.into(),
            mapper_body,
        })),
    })
}

fn string(s: &str) -> Expr {
    Expr::StringLiteral(s.into())
}

fn ch(s: char) -> Expr {
    Expr::CharLiteral(s)
}

fn int(i: u64) -> Expr {
    Expr::IntLiteral(i)
}

fn float(f: f64) -> Expr {
    Expr::DecimalLiteral(f)
}

fn boolean(b: bool) -> Expr {
    Expr::BooleanLiteral(b)
}

fn sint(i: i64) -> Expr {
    Expr::SignedIntLiteral(i)
}

fn bin(bytes: &[u8]) -> Expr {
    Expr::BinaryLiteral(bytes.to_owned())
}

fn with_span<T>(start: usize, end: usize, value: T) -> WithSpan<T> {
    WithSpan {
        span: Span {
            start,
            end,
        },
        value,
    }
}

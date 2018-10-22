use ::interpreter::ast::{
    Expr, FunctionCall, FunctionMapper, GenType, MacroArgument, MacroDef, Program,
};
use ::interpreter::grammar::ExprParser;
use ::interpreter::parser::parse_program;
use IString;

#[test]
fn parses_boolean_literal_false_token() {
    let result = ExprParser::new().parse(r#"false"#);
    assert_eq!(Ok(boolean(false)), result);
}

#[test]
fn parses_boolean_literal_true_token() {
    let result = ExprParser::new().parse(r#"true"#);
    assert_eq!(Ok(boolean(true)), result);
}

#[test]
fn parses_unsigned_int_literal_token() {
    let result = ExprParser::new().parse(r#"1234"#);
    assert_eq!(Ok(int(1234)), result);
}

#[test]
fn parses_unsigned_int_hex_literal_token() {
    let result = ExprParser::new().parse("0xFF");
    assert_eq!(Ok(int(255)), result);
}

#[test]
fn parses_signed_int_literal_negative_token() {
    let result = ExprParser::new().parse(r#"-1234"#);
    assert_eq!(Ok(sint(-1234)), result);
}

#[test]
fn parses_signed_int_literal_positive_token() {
    let result = ExprParser::new().parse(r#"+1234"#);
    assert_eq!(Ok(sint(1234)), result);
}

#[test]
fn parses_string_literal_with_escaped_quotes() {
    let result = ExprParser::new().parse(r#""some\"str""#);
    assert_eq!(Ok(string(r#"some"str"#)), result);
}

#[test]
fn parses_decimal_literal_token() {
    let result = ExprParser::new().parse(r#"123.45"#);
    assert_eq!(Ok(float(123.45)), result);
}

#[test]
fn parses_decimal_literal_token_with_negative_sign() {
    let result = ExprParser::new().parse(r#"-123.45"#);
    assert_eq!(Ok(float(-123.45)), result);
}

#[test]
fn parses_decimal_literal_token_with_positive_sign() {
    let result = ExprParser::new().parse(r#"+123.45"#);
    assert_eq!(Ok(float(123.45)), result);
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
            Expr::StringLiteral("foo".into()),
            Expr::IntLiteral(55),
            Expr::DecimalLiteral(12.5),
        ],
        mapper: None,
    };
    assert_eq!(Ok(Expr::Function(expected)), result);
}

#[test]
fn parses_function_call_with_zero_arguments() {
    let result = ExprParser::new().parse("fun_name()");
    let expected = FunctionCall {
        function_name: "fun_name".into(),
        args: Vec::new(),
        mapper: None,
    };
    assert_eq!(Ok(Expr::Function(expected)), result);
}

#[test]
fn parses_nested_function_calls() {
    let result = ExprParser::new().parse(r#"fun1("foo", fun2(12.5, fun3(111)), "bar")"#);
    let expected = fun(
        "fun1",
        vec![
            string("foo"),
            fun("fun2", vec![float(12.5), fun("fun3", vec![int(111)])]),
            string("bar"),
        ],
    );
    assert_eq!(Ok(expected), result);
}

#[test]
fn parses_mapped_function_call() {
    let input = r#"fun1("foo", 7) {mapper_arg ->
        inner(mapper_arg, mapper_arg)
    }"#;
    let result = ExprParser::new().parse(input);
    let expected = mfun(
        "fun1",
        vec![string("foo"), int(7)],
        "mapper_arg",
        fun(
            "inner",
            vec![arg_usage("mapper_arg"), arg_usage("mapper_arg")],
        ),
    );
    assert_eq!(Ok(expected), result);
}

#[test]
fn parses_mapped_function_call_withou_args() {
    let input = r#"fun1() {mapper_arg ->
        inner(mapper_arg, mapper_arg)
    }"#;
    let result = ExprParser::new().parse(input);
    let expected = mfun(
        "fun1",
        Vec::new(),
        "mapper_arg",
        fun(
            "inner",
            vec![arg_usage("mapper_arg"), arg_usage("mapper_arg")],
        ),
    );
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
            MacroDef {
                name: s("wtf"),
                args: vec![MacroArgument {
                    name: s("count"),
                    arg_type: GenType::Uint,
                }],
                body: Expr::Function(FunctionCall {
                    function_name: s("asciiString"),
                    args: vec![arg_usage("count")],
                    mapper: None,
                }),
                doc_comments: "comment 1".to_owned(),
            },
            MacroDef {
                name: s("foo"),
                args: Vec::new(),
                body: Expr::Function(FunctionCall {
                    function_name: s("wtf"),
                    args: vec![Expr::Function(FunctionCall {
                        function_name: s("uint"),
                        args: vec![Expr::IntLiteral(0), Expr::IntLiteral(9)],
                        mapper: None,
                    })],
                    mapper: None,
                }),
                doc_comments: "comment 2\ncomment 3".to_owned(),
            },
        ],
        expr: Some(Expr::Function(FunctionCall {
            function_name: s("foo"),
            args: Vec::new(),
            mapper: None,
        })),
    };

    let actual = parse_program("test input".into(), input).expect("failed to parse input");
    assert_eq!(expected, actual);
}

#[test]
fn parses_bin_literal() {
    let input = "[ 0x00,0xff, 0x01]";
    let expected_output = bin(&[0x00, 0xff, 0x01]);
    let actual = ExprParser::new().parse(input).expect("failed to parse");
    assert_eq!(expected_output, actual);
}

#[test]
fn parses_empty_bin_literal() {
    let input = "[ ]";
    let expected_output = bin(&[]);
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
    let actual = ExprParser::new().parse(to_parse);
    assert_eq!(Ok(Expr::StringLiteral(s(expected))), actual);
}

fn char_literal_test(to_parse: &str, expected: char) {
    let actual = ExprParser::new().parse(to_parse);
    assert_eq!(Ok(ch(expected)), actual);
}

fn fun(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Function(FunctionCall {
        function_name: name.into(),
        args,
        mapper: None,
    })
}

fn mfun(name: &str, args: Vec<Expr>, mapper_arg_name: &str, mapper_body: Expr) -> Expr {
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

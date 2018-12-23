use crate::interpreter::{Interpreter, UnreadSource};
use crate::program::Runner;
use crate::writer::DataGenOutput;
use crate::ProgramContext;
use failure::Error;

#[test]
fn signed_integer_functions() {
    let input = r#"int(-9, +7)"#;
    let expected_output = "-6";
    test_program_success(1, input, expected_output);
}

#[test]
fn declare_and_use_functions() {
    let expected_output = "aw6OqR822CZggJ42f1aT0";
    let input = r#"
        def foo(len: Uint) = ascii_alphanumeric_chars(len);
        def bar() = foo(7);

        bar()
    "#;
    test_program_success(3, input, expected_output);
}

#[test]
fn use_binary_functions() {
    let program = r##"
        def foo() = select([0x01], [0x02, 0x03], [0x04, 0x05, 0x06]);

        def boo(len: Uint) = len() { i ->
            concat(repeat_delimited(i, [0xFF], [0x00]), [0xAA], repeat(i, foo()))
        };
        boo(3)
        "##;
    let expected = &[0xff, 0x00, 0xff, 0x00, 0xff, 0xAA, 0x01, 0x01, 0x01];
    assert_bin_output_is_expected(program, expected);
}

#[test]
fn stable_select_a_generator() {
    let input = r#"stable_select(select("a", "b"), select("c", "d"))"#;
    let expected_output = "aabbbbbaba";
    test_program_success(10, input, expected_output);
}

#[test]
fn use_std_boolean_function() {
    let expected_output = "truetruetrue";
    let input = r#"boolean()"#;
    test_program_success(3, input, expected_output);
}

#[test]
fn declare_and_use_function_with_mapper() {
    let input = r#"
        def repeat_words(times: Uint) = times() { num ->
            concat(to_string(num), " : ", repeat(num, ascii_alphanumeric_chars(5) { word ->
                repeat(num, concat(word, "\n"))
            }))
        };

        def count() = uint(2, 5);

        concat(repeat_words(count()), repeat_words(count()))
    "#;
    let expected = "2 : w6OqR\nw6OqR\n822CZ\n822CZ\n3 : gJ42f\ngJ42f\n1aT0X\n1aT0X\n";
    test_program_success(1, input, expected);
}

#[test]
fn pass_mapped_function_as_function_argument() {
    let input = r#"
        def compare_words(word_fun: String) = 
            repeat(3, concat(word_fun, " != ", word_fun, "\n"));

        compare_words(ascii_alphanumeric_chars(1) { w -> repeat_delimited(3, w, ", ") } )
    "#;
    let expected = "a, a, a != w, w, w\n6, 6, 6 != O, O, O\nq, q, q != R, R, R\n";
    test_program_success(1, input, expected);
}

#[test]
fn mapping_a_mapped_function() {
    let input = r#"
        def compare_words(word_fun: String) = 
            repeat(3, word_fun() {word -> 
                concat(single_quote(word), " == ", single_quote(word), "\n")
            });
        
        compare_words(ascii_alphanumeric_chars(1) { w -> repeat_delimited(3, w, "_") } )
    "#;
    let expected = "\'a_a_a\' == \'a_a_a\'\n\'w_w_w\' == \'w_w_w\'\n\'6_6_6\' == \'6_6_6\'\n";
    test_program_success(1, input, expected);
}

#[test]
fn calling_a_function_with_module_name() {
    let lib1 = r##"
        def foo() = "lib1foo";
    "##;

    let lib2 = r##"
        def foo() = "lib2foo";
    "##;

    let mut runner = Runner::new(
        1,
        "lib2.foo()".to_owned(),
        create_context(),
        Interpreter::new(),
    );
    runner
        .add_library(UnreadSource::Builtin("lib1", lib1))
        .unwrap();
    runner
        .add_library(UnreadSource::Builtin("lib2", lib2))
        .unwrap();

    let result = run_to_string(runner);
    assert_eq!("lib2foo", &result);

    let mut runner = Runner::new(
        1,
        "lib1.foo()".to_owned(),
        create_context(),
        Interpreter::new(),
    );
    runner
        .add_library(UnreadSource::Builtin("lib1", lib1))
        .unwrap();
    runner
        .add_library(UnreadSource::Builtin("lib2", lib2))
        .unwrap();

    let result = run_to_string(runner);
    assert_eq!("lib1foo", &result);
}

#[test]
fn adding_a_library_that_defines_two_functions_with_the_same_signature_returns_error() {
    let lib = r##"
    # the first foo function
    def foo(i: Uint) = ascii_alphanumeric_chars(i);

    # the second foo function
    def foo(i: Uint) = unicode_string(i);
    "##;

    let mut runner = Runner::new(1, "bar()".to_owned(), create_context(), Interpreter::new());
    let result = runner.add_library(lib.to_owned());
    assert!(result.is_err());
    let error = result.unwrap_err();
    let err_str = format!("{}", error);
    assert!(
        err_str.contains("Module 'default' contains multiple functions with the same signature"),
        "Error string was incorrect. Actual error: {}",
        err_str
    );
}

const RAND_SEED: &[u8; 16] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

pub fn create_context() -> ProgramContext {
    ProgramContext::from_seed(*RAND_SEED, crate::verbosity::NORMAL)
}
pub fn run_program(iterations: u64, program: &str) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    {
        let mut output = DataGenOutput::new(&mut out);
        let mut prog = Runner::new(
            iterations,
            program.to_owned(),
            create_context(),
            Interpreter::new(),
        );
        prog.add_std_lib();
        prog.run(&mut output).map_err(|error| {
            format_err!("Failed to run program. Eror: {}", error)
        })?;
    }

    Ok(out)
}

fn run_to_string(runner: Runner) -> String {
    let mut out = Vec::new();
    {
        let mut output = DataGenOutput::new(&mut out);
        runner.run(&mut output).expect("failed to run program");
    }
    String::from_utf8(out).expect("program results were not valid utf8")
}

pub fn assert_bin_output_is_expected(program: &str, expected: &[u8]) {
    let results = run_program(1, program).expect("Failed to run program");
    assert_eq!(results.as_slice(), expected);
}

pub fn test_program_success(iterations: u64, input: &str, expected_output: &str) {
    let results = run_program(iterations, input).expect("Failed to run program");
    let as_str = String::from_utf8(results).expect("program results were not valid utf8");
    if expected_output != as_str.as_str() {
        panic!(
            "Incorrect program output, expected: '{}', actual: '{}', actual_debug: '{:?}'",
            expected_output, as_str, as_str
        );
    }
}

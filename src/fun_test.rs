use super::Program;
use failure::Error;
use v2::ProgramContext;
use writer::DataGenOutput;

#[test]
fn generate_ascii_strings() {
    let expected_output = "w6U9vomgJ4gxen0XO";
    let input = "alphanumeric_string(uint(0, 10))";
    test_program_success(4, input, expected_output);
}

// #[test]
// fn generate_unicode_strings() {
//     let expected_output = "栈 \u{8cc}겙\u{fd4f}긧ﶩ鵝ᣧ蹈澨ꇦ笲";
//     let input = "bmp_string(uint(0, 10))";
//     test_program_success(4, input, expected_output);
// }

#[test]
fn signed_integer_functions() {
    let input = r#"int(-9, +7)"#;
    let expected_output = "-9";
    test_program_success(1, input, expected_output);
}

#[test]
fn declare_and_use_functions() {
    let expected_output = "aw6IU9vomgJ42f1aT0XOE";
    let input = r#"
        def foo(len: Uint) = alphanumeric_string(len);
        def bar() = foo(7);

        bar()
    "#;
    test_program_success(3, input, expected_output);
}

#[test]
fn stable_select_a_generator() {
    let input = r#"stable_select(select("a", "b"), select("c", "d"))"#;
    let expected_output = "aabbbbbbaa";
    test_program_success(10, input, expected_output);
}

#[test]
fn use_custom_string_function() {
    let input = r#"
        def consonants() = select('b', 'c', 'd', 'f', 'g', 'h', 'j', 'k', 
                'l', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z');
        def vowels() = select('a', 'e', 'i', 'o', 'u');

        def chars() = select(vowels(), consonants());

        string(10, chars())
    "#;
    let expected_output = "ausjmhaevg";
    test_program_success(1, input, expected_output);
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
            concat(to_string(num), " : ", repeat(num, alphanumeric_string(5) { word ->
                repeat(num, concat(word, "\n"))
            }))
        };

        def count() = uint(2, 5);

        concat(repeat_words(count()), repeat_words(count()))
    "#;
    let expected = "2 : w6IU9\nw6IU9\nvomgJ\nvomgJ\n4 : 2f1aT\n2f1aT\n2f1aT\n0XOET\n0XOET\n0XOET\n9Vk0R\n9Vk0R\n9Vk0R\n";
    test_program_success(1, input, expected);
}

#[test]
fn pass_mapped_function_as_function_argument() {
    let input = r#"
        def compare_words(word_fun: String) = 
            repeat(3, concat(word_fun, " != ", word_fun, "\n"));

        compare_words(alphanumeric_string(1) { w -> repeat_delimited(3, w, ", ") } )
    "#;
    let expected = "a, a, a != w, w, w\n6, 6, 6 != I, I, I\nU, U, U != 9, 9, 9\n";
    test_program_success(1, input, expected);
}

#[test]
fn mapping_a_mapped_function() {
    let input = r#"
        def compare_words(word_fun: String) = 
            repeat(3, word_fun() {word -> 
                concat(single_quote(word), " == ", single_quote(word), "\n")
            });
        
        compare_words(alphanumeric_string(1) { w -> repeat_delimited(3, w, "_") } )
    "#;
    let expected = "\'a_a_a\' == \'a_a_a\'\n\'w_w_w\' == \'w_w_w\'\n\'6_6_6\' == \'6_6_6\'\n";
    test_program_success(1, input, expected);
}

const RAND_SEED: &[u8; 16] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

fn run_program(iterations: u64, program: &str) -> Result<Vec<u8>, Error> {
    let rng = ProgramContext::from_seed(*RAND_SEED);

    let mut out = Vec::new();
    {
        let mut output = DataGenOutput::new(&mut out);
        let mut prog = Program::new(2, iterations, program.to_owned(), rng);
        prog.add_std_lib();
        prog.run(&mut output)?;
    }

    Ok(out)
}

fn test_program_success(iterations: u64, input: &str, expected_output: &str) {
    let results = run_program(iterations, input).expect("Failed to run program");
    let as_str = String::from_utf8(results).expect("program results were not valid utf8");
    if expected_output != as_str.as_str() {
        panic!(
            "Incorrect program output, expected: '{}', actual: '{}', actual_debug: '{:?}'",
            expected_output, as_str, as_str
        );
    }
}

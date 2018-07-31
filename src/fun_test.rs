use super::Program;
use failure::Error;
use generator::DataGenRng;
use rand::SeedableRng;
use writer::DataGenOutput;

#[test]
fn generate_ascii_strings() {
    let expected_output = "aACrrnGjOTedJsRy";
    let input = "alphanumeric_string(uint(0, 10))";
    test_program_success(4, input, expected_output);
}

#[test]
fn declare_and_use_macros() {
    let expected_output = "DPaADCI2CrrnGjOTboedJ";
    let input = r#"
        def foo(len: Uint) = alphanumeric_string(len());
        def bar() = foo(7);

        bar()
    "#;
    test_program_success(3, input, expected_output);
}

#[test]
fn use_custom_string_function() {
    let input = r#"
        def consonants() = one_of('b', 'c', 'd', 'f', 'g', 'h', 'j', 'k', 
                'l', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z');
        def vowels() = one_of('a', 'e', 'i', 'o', 'u');

        def chars() = either(vowels, consonants);

        string(10, chars)
    "#;
    let expected_output = "ausjmhpije";
    test_program_success(1, input, expected_output);
}

#[test]
fn use_easy_csv_function() {
    let input = "easy_csv(2, 3)";
    let expected_output = r#""DCI2","rnGjOTboedJsRyC2F59PJ1KOiibFmf9eT8P"
"6856658967277113641","true"
"true","p5xK7LZAhglu"
"true","16469683845218708375"
"#;
    test_program_success(1, input, expected_output);
}

#[test]
fn use_std_boolean_function() {
    let expected_output = "truetruetrue";
    let input = r#"boolean(1.0)"#;
    test_program_success(3, input, expected_output);
}

#[test]
fn declare_and_use_function_with_mapper() {
    let input = r#"
        def repeat_words(times: Uint) = times() { num ->
            concat(num, " : ", repeat(num, alphanumeric_string(5) { word ->
                repeat(num, trailing_newline(word))
            }))
        };

        def count() = uint(2, 5);

        concat(repeat_words(count), repeat_words(count))
    "#;
    let expected = "2 : aADCI\naADCI\n2Crrn\n2Crrn\n3 : OTboe\nOTboe\nOTboe\ndJsRy\ndJsRy\ndJsRy\nC2F59\nC2F59\nC2F59\n";
    test_program_success(1, input, expected);
}

#[test]
fn pass_mapped_function_as_function_argument() {
    let input = r#"
        def compare_words(word_fun: String) = 
            repeat(3, concat(word_fun, " != ", word_fun, "\n"));

        compare_words(alphanumeric() { w -> repeat_delimited(3, w, ", ") } )
    "#;
    let expected = "D, D, D != P, P, P\na, a, a != A, A, A\nD, D, D != C, C, C\n";
    test_program_success(1, input, expected);
}

#[test]
fn mapping_a_mapped_function() {
    let input = r#"
        def compare_words(word_fun: String) = 
            repeat(3, word_fun() {word -> 
                concat(single_quote(word), " == ", single_quote(word), "\n")
            });
        
        compare_words(alphanumeric() { w -> repeat_delimited(3, w, "_") } )
    "#;
    let expected = "'D_D_D' == 'D_D_D'\n'P_P_P' == 'P_P_P'\n'a_a_a' == 'a_a_a'\n";
    test_program_success(1, input, expected);
}

const RAND_SEED: &[u8; 16] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

fn run_program(iterations: u64, program: &str) -> Result<Vec<u8>, Error> {
    let rng = DataGenRng::from_seed(*RAND_SEED);

    let mut out = Vec::new();
    {
        let mut output = DataGenOutput::new(&mut out);
        let mut prog = Program::new(2, iterations, program.to_owned(), rng);
        prog.add_library(::libraries::STANDARD_LIB).expect("failed to eval std library");
        prog.run(&mut output)?;
    }

    Ok(out)
}

fn test_program_success(iterations: u64, input: &str, expected_output: &str) {
    let results = run_program(iterations, input).expect("Failed to run program");
    let as_str = String::from_utf8(results).expect("program results were not valid utf8");
    assert_eq!(expected_output, as_str.as_str());
}

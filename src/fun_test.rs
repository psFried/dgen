use rand::SeedableRng;
use generator::DataGenRng;
use writer::DataGenOutput;
use super::Program;
use failure::Error;


#[test]
fn generate_ascii_strings() {
    let expected_output = "aACrrnGjOTedJsRy";
    let input = "asciiString(uint(0, 10))";
    test_program_success(4, input, expected_output)
}

const RAND_SEED: &[u8; 16] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

fn run_program(iterations: u64, program: &str) -> Result<Vec<u8>, Error> {
    let rng = DataGenRng::from_seed(*RAND_SEED);
    
    let mut out = Vec::new();
    {
        let output = DataGenOutput::new(&mut out);
        let prog = Program::new(2, iterations, program.to_owned(), rng, output);
        prog.run()?;
    }

    Ok(out)
}

fn test_program_success(iterations: u64, input: &str, expected_output: &str) {
    let results = run_program(iterations, input).expect("Failed to run program");
    let as_str = String::from_utf8(results).expect("program results were not valid utf8");
    assert_eq!(expected_output, as_str.as_str());
}
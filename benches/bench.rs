#[macro_use]
extern crate criterion;
extern crate dgen;
extern crate encoding;
extern crate rand;

use criterion::{Criterion, Fun};
use dgen::interpreter::UnreadSource;
use dgen::{DataGenOutput, Interpreter, ProgramContext};
use rand::prng::XorShiftRng;
use rand::{FromEntropy, Rng};
use std::fmt;
use std::io::Write;

const SEED: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
const OUT_CAPACITY: usize = 32 * 1024;

fn create_string_benchmark<I: fmt::Debug + 'static>(program: &'static str) -> Fun<I> {
    let mut interpreter = Interpreter::new();
    interpreter.add_std_lib();
    let compiled = interpreter
        .eval(UnreadSource::Builtin("test", program))
        .unwrap();
    let mut context = ProgramContext::from_seed(SEED, ::dgen::verbosity::NORMAL);
    let mut out = Vec::with_capacity(OUT_CAPACITY);

    Fun::new(program, move |b, _| {
        b.iter(|| {
            out.clear();
            let mut real_out = DataGenOutput::new(&mut out);
            compiled.write_value(&mut context, &mut real_out).unwrap();
        })
    })
}

fn homogeneous_string_benches(c: &mut Criterion) {
    let functions = vec![
        create_string_benchmark("lowercase_ascii_string(16)"),
        create_string_benchmark("lowercase_ascii_string(128)"),
        create_string_benchmark("lowercase_ascii_string(1024)"),
    ];

    c.bench_functions("heterogeneous_strings", functions, ());
}

fn heterogeneous_string_benches(c: &mut Criterion) {
    let functions = vec![
        create_string_benchmark("alphanumeric_string(16)"),
        create_string_benchmark("alphanumeric_string(128)"),
        create_string_benchmark("alphanumeric_string(1024)"),
    ];

    c.bench_functions("heterogeneous_strings", functions, ());
}

#[derive(Clone)]
struct RandomBytes(Vec<u8>);
impl RandomBytes {
    fn with_length(len: usize) -> RandomBytes {
        let mut rng = XorShiftRng::from_entropy();
        let mut vec = vec![0; len];
        rng.fill(vec.as_mut_slice());
        RandomBytes(vec)
    }
    fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}
impl fmt::Debug for RandomBytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RandomBytes(length={})", self.0.len())
    }
}

fn writer_benches(c: &mut Criterion) {
    let mut bytes = Vec::with_capacity(8 * 1024);

    let sizes = &[1, 16, 256, 4096];
    let inputs = sizes
        .iter()
        .cloned()
        .map(RandomBytes::with_length)
        .collect::<Vec<RandomBytes>>();

    c.bench_function_over_inputs(
        "datagen_output",
        move |bencher, input| {
            bencher.iter(|| {
                bytes.clear();
                let mut out = DataGenOutput::new(&mut bytes);
                out.write_bytes(input.as_slice()).unwrap()
            });
        },
        inputs.clone(),
    );

    // So we can compare the two and see how much overhead is introduced by DataGenOutput
    let mut raw_bytes = Vec::with_capacity(8 * 1024);
    c.bench_function_over_inputs(
        "raw_vec",
        move |bencher, input| {
            bencher.iter(|| {
                raw_bytes.clear();
                raw_bytes.write_all(input.as_slice()).unwrap()
            })
        },
        inputs,
    );
}

criterion_group!(
    benches,
    writer_benches,
    heterogeneous_string_benches,
    homogeneous_string_benches
);
criterion_main!(benches);

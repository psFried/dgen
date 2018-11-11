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

macro_rules! make_string_bench {
    ($str_len:tt) => {{
        let program = stringify!(string($str_len, ascii_alphanumeric_char()));
        let mut interpreter = Interpreter::new();
        interpreter.add_std_lib();
        let compiled = interpreter
            .eval(UnreadSource::Builtin("test", program))
            .unwrap();
        let mut context = ProgramContext::from_seed(SEED, ::dgen::verbosity::NORMAL);
        let mut out = Vec::with_capacity(OUT_CAPACITY);

        let fun_name = stringify!(ascii_string_$str_len);
        Fun::new(fun_name, move |b, _| {
            b.iter(|| {
                out.clear();
                let mut real_out = DataGenOutput::new(&mut out);
                compiled.write_value(&mut context, &mut real_out).unwrap();
            })
        })
    }};
}
fn string_gen_benches(c: &mut Criterion) {
    let fun_1000 = make_string_bench!(1000);
    let fun_100 = make_string_bench!(100);
    let fun_10 = make_string_bench!(10);

    let functions = vec![fun_10, fun_100, fun_1000];

    c.bench_functions("ascii_alphanumeric_string", functions, ());
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

    let sizes = &[1, 8, 24, 256, 4096];
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

criterion_group!(benches, writer_benches, string_gen_benches);
criterion_main!(benches);

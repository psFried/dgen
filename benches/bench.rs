#[macro_use]
extern crate criterion;
extern crate dgen;
extern crate encoding;
extern crate rand;

use criterion::{Criterion, Fun};
use dgen::interpreter::UnreadSource;
use dgen::{DataGenOutput, Interpreter, ProgramContext};
use encoding::{ByteWriter, EncoderTrap, Encoding};
use rand::prng::XorShiftRng;
use rand::{FromEntropy, Rand, Rng};
use std::fmt;
use std::io::{self, Write};

/*
fn test_char(c: char) -> usize {
    let mut bytes = [0u8; 8];
    let s = c.encode_utf8(&mut bytes[..]);
    s.len()
}

fn test_fmt_char(c: char) -> usize {
    let mut bytes = [0u8; 8];
    let mut wtf = &mut bytes[..];
    let mut out = DataGenOutput::new(&mut wtf);
    out.write(&c).unwrap() as usize
}

fn test_dgen_encode_char(c: char) -> usize {
    let mut bytes = [0u8; 8];
    let mut wtf = &mut bytes[..];
    let mut out = DataGenOutput::new(&mut wtf);
    out.write_char(c).unwrap() as usize
}

fn test_utf8_string_encoding(value: &str) -> usize {
    let encoding = ::encoding::all::UTF_8;
    let mut bytes = [0u8; 8];
    let mut writer = SliceByteWriter(&mut bytes[..], 0);
    encoding
        .encode_to(value, EncoderTrap::Strict, &mut writer)
        .unwrap();
    writer.1
}
fn test_utf8_string_copy(value: &str) -> usize {
    let encoding = ::encoding::all::UTF_8;
    let mut bytes = [0u8; 8];
    let mut wtf = &mut bytes[..];
    let mut writer = DataGenOutput::new(&mut wtf);
    writer.write_bytes(value.as_bytes()).unwrap() as usize
}

fn string_bench_ascii(c: &mut Criterion) {
    let encode_fun = Fun::new("encode_utf8_string", |b, i| {
        b.iter(|| test_utf8_string_encoding(*i))
    });
    let copy_fun = Fun::new("copy_utf8_string", |b, i| {
        b.iter(|| test_utf8_string_copy(*i))
    });
    let funs = vec![encode_fun, copy_fun];

    c.bench_functions("write_ascii_strings", funs, "a");
}
fn string_bench_unicode(c: &mut Criterion) {
    let encode_fun = Fun::new("encode_utf8_string", |b, i| {
        b.iter(|| test_utf8_string_encoding(*i))
    });
    let copy_fun = Fun::new("copy_utf8_string", |b, i| {
        b.iter(|| test_utf8_string_copy(*i))
    });
    let funs = vec![encode_fun, copy_fun];

    c.bench_functions("write_unicode_strings", funs, "\u{1F4A9}");
}

fn char_benchmark_ascii(c: &mut Criterion) {
    let encode_fun = Fun::new("encode_ascii_utf8", |b, i| b.iter(|| test_char(*i)));
    let fmt_fun = Fun::new("fmt_ascii_char", |b, i| b.iter(|| test_fmt_char(*i)));
    let dgen_encode_fun = Fun::new("dgen_encode_ascii_char", |b, i| {
        b.iter(|| test_dgen_encode_char(*i))
    });

    c.bench_functions(
        "write_chars",
        vec![encode_fun, fmt_fun, dgen_encode_fun],
        'a',
    );
}

fn char_benchmark_unicode(c: &mut Criterion) {
    let encode_fun = Fun::new("encode_unicode_utf8", |b, i| b.iter(|| test_char(*i)));
    let dgen_encode_fun = Fun::new("dgen_encode_unicode_char", |b, i| {
        b.iter(|| test_dgen_encode_char(*i))
    });
    let fmt_fun = Fun::new("fmt_unicode_char", |b, i| b.iter(|| test_fmt_char(*i)));

    c.bench_functions(
        "write_chars",
        vec![encode_fun, fmt_fun, dgen_encode_fun],
        '\u{1F4A9}',
    );
}
*/

const SEED: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
const OUT_CAPACITY: usize = 32 * 1024;

macro_rules! make_string_bench {
    ($str_len:tt) => {{
        let program = stringify!(string($str_len, ascii_alphanumeric_char()));
        let mut interpreter = ::dgen::Interpreter::new();
        interpreter.add_std_lib();
        let compiled = interpreter
            .eval(UnreadSource::Builtin("test", program))
            .unwrap();
        let mut context = ProgramContext::from_seed(SEED, ::dgen::verbosity::NORMAL);
        let mut out = Vec::with_capacity(OUT_CAPACITY);

        let fun_name = stringify!(ascii_string_$str_len);
        Fun::new(fun_name, move |b, i| {
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
    let mut rng = XorShiftRng::from_entropy();
    let mut bytes = Vec::with_capacity(8 * 1024);

    let sizes = &[1, 3, 8, 24, 128, 512, 4096];
    let inputs = sizes
        .iter().cloned()
        .map(RandomBytes::with_length).collect::<Vec<RandomBytes>>();

    c.bench_function_over_inputs(
        "datagen_output",
        move |bencher, input| {
            bencher.iter(|| {
                bytes.clear();
                let mut out = DataGenOutput::new(&mut bytes);
                out.write_bytes(input.as_slice()).unwrap()
            });
        },
        inputs,
    );
}

criterion_group!(benches, string_gen_benches, writer_benches);
//criterion_group!(benches, char_benchmark_ascii, char_benchmark_unicode, string_bench_ascii, string_bench_unicode);
criterion_main!(benches);

/*
struct SliceByteWriter<'a>(&'a mut [u8], usize);

impl<'a> Write for SliceByteWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len();
        let dst = &mut self.0[self.1..(self.1 + len)];
        dst.copy_from_slice(buf);
        self.1 += len;
        Ok(len)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> ByteWriter for SliceByteWriter<'a> {
    fn write_byte(&mut self, byte: u8) {
        self.0[self.1] = byte;
        self.1 += 1;
    }
    fn write_bytes(&mut self, bytes: &[u8]) {
        let len = bytes.len();
        let mut dst = &mut self.0[self.1..(self.1 + len)];
        dst.copy_from_slice(bytes);
        self.1 += len;
    }
}
*/

/*
with write_char:
write_chars/encode_utf8 time:   [379.78 ps 380.75 ps 381.92 ps]
Found 13 outliers among 100 measurements (13.00%)
  3 (3.00%) high mild
  10 (10.00%) high severe
write_chars/fmt_char    time:   [13.938 ns 13.982 ns 14.026 ns]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe


with write_string:
write_chars/encode_utf8 time:   [378.76 ps 379.67 ps 380.85 ps]
                        change: [-1.0503% -0.2328% +0.6291%] (p = 0.59 > 0.05)
                        No change in performance detected.
Found 21 outliers among 100 measurements (21.00%)
  5 (5.00%) high mild
  16 (16.00%) high severe
write_chars/fmt_char    time:   [38.195 ns 38.917 ns 39.755 ns]
                        change: [+171.68% +176.20% +180.69%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe

write_chars/encode_utf8 time:   [694.50 ps 695.93 ps 697.55 ps]
                        change: [+80.488% +81.674% +82.738%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) high mild
  7 (7.00%) high severe
write_chars/fmt_char    time:   [40.070 ns 40.418 ns 40.823 ns]
                        change: [+3.4990% +5.5662% +7.5698%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe

Writing as strings:
write_ascii_strings/encode_utf8_string
                        time:   [21.091 ns 21.126 ns 21.166 ns]
Found 16 outliers among 100 measurements (16.00%)
  7 (7.00%) high mild
  9 (9.00%) high severe
write_ascii_strings/copy_utf8_string
                        time:   [4.5553 ns 4.5670 ns 4.5808 ns]
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) high mild
  8 (8.00%) high severe

write_unicode_strings/encode_utf8_string
                        time:   [23.000 ns 23.077 ns 23.175 ns]
Found 20 outliers among 100 measurements (20.00%)
  5 (5.00%) high mild
  15 (15.00%) high severe
write_unicode_strings/copy_utf8_string
                        time:   [5.3045 ns 5.3170 ns 5.3323 ns]
Found 19 outliers among 100 measurements (19.00%)
  4 (4.00%) high mild
  15 (15.00%) high severe
*/

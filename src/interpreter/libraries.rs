use ::interpreter::Source;

macro_rules! include_lib {
    ($name:expr, $filename:expr) => {
        {
            const LIB_CONTENT: &'static str = include_str!(concat!("../../dgen_libs/", $filename));
            &Source::Builtin($name, LIB_CONTENT)
        }
    };
}

const CHARS: &'static Source = include_lib!("std.chars", "std/chars.dgen");
const STRINGS: &'static Source = include_lib!("std.strings", "std/strings.dgen");
const NUMBERS: &'static Source = include_lib!("std.numbers", "std/numbers.dgen");
const BOOLEAN: &'static Source = include_lib!("std.boolean", "std/boolean.dgen");
const REPEATS: &'static Source = include_lib!("std.repeats", "std/repeats.dgen");


pub const STDLIBS: &[&Source] = &[
    CHARS,
    STRINGS,
    NUMBERS,
    BOOLEAN,
    REPEATS,
];

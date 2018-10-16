use v2::interpreter::Source;

macro_rules! include_lib {
    ($name:expr) => {
        {
            const LIB_CONTENT: &'static str = include_str!(concat!("../../../pgen_libs/", $name));
            &Source::Builtin($name, LIB_CONTENT)
        }
    };
}

const CHARS: &'static Source = include_lib!("std/chars.pgen");
const STRINGS: &'static Source = include_lib!("std/strings.pgen");
const NUMBERS: &'static Source = include_lib!("std/numbers.pgen");
const BOOLEAN: &'static Source = include_lib!("std/boolean.pgen");
const REPEATS: &'static Source = include_lib!("std/repeats.pgen");


pub const STDLIBS: &[&Source] = &[
    CHARS,
    STRINGS,
    NUMBERS,
    BOOLEAN,
    REPEATS,
];
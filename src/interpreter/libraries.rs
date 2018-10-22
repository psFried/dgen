use ::interpreter::Source;

macro_rules! include_lib {
    ($name:expr) => {
        {
            const LIB_CONTENT: &'static str = include_str!(concat!("../../dgen_libs/", $name));
            &Source::Builtin($name, LIB_CONTENT)
        }
    };
}

const CHARS: &'static Source = include_lib!("std/chars.dgen");
const STRINGS: &'static Source = include_lib!("std/strings.dgen");
const NUMBERS: &'static Source = include_lib!("std/numbers.dgen");
const BOOLEAN: &'static Source = include_lib!("std/boolean.dgen");
const REPEATS: &'static Source = include_lib!("std/repeats.dgen");


pub const STDLIBS: &[&Source] = &[
    CHARS,
    STRINGS,
    NUMBERS,
    BOOLEAN,
    REPEATS,
];

use crate::interpreter::UnreadSource;

macro_rules! include_lib {
    ($name:expr, $filename:expr) => {{
        const LIB_CONTENT: &'static str = include_str!(concat!("../../dgen_libs/", $filename));
        &UnreadSource::Builtin($name, LIB_CONTENT)
    }};
}

const STRINGS: &'static UnreadSource = include_lib!("std.strings", "std/strings.dgen");
const NUMBERS: &'static UnreadSource = include_lib!("std.numbers", "std/numbers.dgen");
const BOOLEAN: &'static UnreadSource = include_lib!("std.boolean", "std/boolean.dgen");
const REPEATS: &'static UnreadSource = include_lib!("std.repeats", "std/repeats.dgen");

pub const STDLIBS: &[&UnreadSource] = &[STRINGS, NUMBERS, BOOLEAN, REPEATS];

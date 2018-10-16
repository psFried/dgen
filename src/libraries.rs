const CHARS: &'static str = include_str!("../pgen_libs/chars.pgen");
const STRINGS: &'static str = include_str!("../pgen_libs/strings.pgen");
const NUMBERS: &'static str = include_str!("../pgen_libs/numbers.pgen");
const BOOLEAN: &'static str = include_str!("../pgen_libs/boolean.pgen");
const REPEATS: &'static str = include_str!("../pgen_libs/repeats.pgen");


pub const STDLIBS: &[&str] = &[
    CHARS,
    STRINGS,
    NUMBERS,
    BOOLEAN,
    REPEATS,
];
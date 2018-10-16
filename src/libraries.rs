pub const CHARS: &'static str = include_str!("../pgen_libs/chars.pgen");
pub const STRINGS: &'static str = include_str!("../pgen_libs/strings.pgen");
pub const NUMBERS: &'static str = include_str!("../pgen_libs/numbers.pgen");


pub const STDLIBS: &[&str] = &[
    CHARS,
    STRINGS,
    NUMBERS,
];
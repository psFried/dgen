mod chars;
mod concat;
mod select;
mod strings;

use v2::FunctionPrototype;

pub const BUILTIN_FNS: &'static [&'static FunctionPrototype] = &[
    self::chars::CHAR_GEN_BUILTIN,
    self::strings::STRING_GEN_BUILTIN,
    self::concat::CONCAT_BUILTIN,
];

mod chars;
mod concat;
mod from_file;
mod numeric;
mod repeat_delim;
mod select;
mod strings;

use v2::FunctionPrototype;

pub const BUILTIN_FNS: &'static [&'static FunctionPrototype] = &[
    self::chars::CHAR_GEN_BUILTIN,
    self::strings::STRING_GEN_BUILTIN,
    self::concat::CONCAT_BUILTIN,
    self::select::SELECT_CHAR_BUILTIN,
    self::select::SELECT_STRING_BUILTIN,
    self::select::SELECT_BOOLEAN_BUILTIN,
    self::select::SELECT_DECIMAL_BUILTIN,
    self::select::SELECT_UINT_BUILTIN,
    self::select::SELECT_INT_BUILTIN,
    self::select::SELECT_BIN_BUILTIN,
    self::repeat_delim::REPEAT_DELIM_BUILTIN,
    self::numeric::UINT_BUILTIN,
    self::numeric::INT_BUILTIN,
    self::numeric::DECIMAL_BUILTIN,
    self::from_file::WORDS_BUILTIN,
    self::from_file::SELECT_FROM_FILE_BUILTIN,
];

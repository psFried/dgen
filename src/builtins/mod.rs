mod bin_length;
mod byte_order;
mod chars;
mod concat;
mod env;
mod from_file;
mod numeric;
mod repeat_delim;
mod select;
mod sequence;
mod strings;
mod to_string;

use crate::interpreter::Module;
use crate::BuiltinFunctionPrototype;

const BUILTIN_FNS: &'static [&'static BuiltinFunctionPrototype] = &[
    self::bin_length::BIN_LENGTH,
    self::byte_order::UINT_LITTLE_ENDIAN,
    self::byte_order::UINT_BIG_ENDIAN,
    self::byte_order::INT_LITTLE_ENDIAN,
    self::byte_order::INT_BIG_ENDIAN,
    self::byte_order::DECIMAL_LITTLE_ENDIAN,
    self::byte_order::DECIMAL_BIG_ENDIAN,
    self::chars::CHAR_GEN_BUILTIN,
    self::env::ENV_VAR,
    self::strings::STRING_GEN_BUILTIN,
    self::strings::STRING_LENGTH_BUILTIN,
    self::strings::STRING_ENCODE_BUILTIN,
    self::concat::CONCAT_BUILTIN,
    self::concat::CONCAT_BIN_BUILTIN,
    self::select::SELECT_CHAR_BUILTIN,
    self::select::SELECT_STRING_BUILTIN,
    self::select::SELECT_BOOLEAN_BUILTIN,
    self::select::SELECT_DECIMAL_BUILTIN,
    self::select::SELECT_UINT_BUILTIN,
    self::select::SELECT_INT_BUILTIN,
    self::select::SELECT_BIN_BUILTIN,
    self::select::STABLE_SELECT_CHAR_BUILTIN,
    self::select::STABLE_SELECT_STRING_BUILTIN,
    self::select::STABLE_SELECT_BOOLEAN_BUILTIN,
    self::select::STABLE_SELECT_DECIMAL_BUILTIN,
    self::select::STABLE_SELECT_UINT_BUILTIN,
    self::select::STABLE_SELECT_INT_BUILTIN,
    self::select::STABLE_SELECT_BIN_BUILTIN,
    self::repeat_delim::REPEAT_DELIM_BUILTIN,
    self::repeat_delim::REPEAT_DELIM_BIN_BUILTIN,
    self::numeric::UINT_BUILTIN,
    self::numeric::INT_BUILTIN,
    self::numeric::DECIMAL_BUILTIN,
    self::from_file::WORDS_BUILTIN,
    self::from_file::SELECT_FROM_FILE_BUILTIN,
    self::to_string::BOOLEAN_TO_STRING_BUILTIN,
    self::to_string::CHAR_TO_STRING_BUILTIN,
    self::to_string::DECIMAL_TO_STRING_BUILTIN,
    self::to_string::INT_TO_STRING_BUILTIN,
    self::to_string::UINT_TO_STRING_BUILTIN,
    self::sequence::CHAR_WRAPPING_SEQ,
    self::sequence::CHAR_SEQ,
    self::sequence::STRING_WRAPPING_SEQ,
    self::sequence::STRING_SEQ,
    self::sequence::BIN_WRAPPING_SEQ,
    self::sequence::BIN_SEQ,
    self::sequence::UINT_WRAPPING_SEQ,
    self::sequence::UINT_SEQ,
    self::sequence::INT_WRAPPING_SEQ,
    self::sequence::INT_SEQ,
    self::sequence::DECIMAL_WRAPPING_SEQ,
    self::sequence::DECIMAL_SEQ,
];


pub fn get_default_builtins_module() -> Module {
    Module::new_builtin(BUILTIN_FNS.iter().map(|fun| *fun))
}
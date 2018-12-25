use crate::IString;
use crate::{
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput,
    DynStringFun, DynUintFun, GenType, ProgramContext, RunnableFunction,
};
use encoding::label::encoding_from_whatwg_label;
use encoding::EncoderTrap;
use failure::Error;
use std::rc::Rc;

#[derive(Debug)]
pub struct StringGenerator {
    length_gen: DynUintFun,
    min_cp_inclusive: DynUintFun,
    max_cp_inclusive: DynUintFun,
}

impl StringGenerator {
    fn gen_char(&self, context: &mut ProgramContext) -> Result<char, Error> {
        let min = self.min_cp_inclusive.gen_value(context)?;
        let max = self.max_cp_inclusive.gen_value(context)?;

        let as_u64 = context.gen_range_inclusive(min, max);

        ::std::char::from_u32(as_u64 as u32).ok_or_else(|| {
            format_err!("Invalid unicode codepoint: {}, generated from range: min_inclusive: {}, max_inclusive: {}", as_u64, min, max)
        })
    }
}

impl RunnableFunction<IString> for StringGenerator {
    fn gen_value(&self, context: &mut ProgramContext) -> Result<IString, Error> {
        let num_iterations = self.length_gen.gen_value(context)? as usize;

        let mut buf = String::with_capacity(num_iterations);

        for _ in 0..num_iterations {
            let c = self.gen_char(context)?;
            buf.push(c);
        }
        Ok(buf.into())
    }

    fn write_value(
        &self,
        context: &mut ProgramContext,
        out: &mut DataGenOutput,
    ) -> Result<(), Error> {
        let num_iterations = self.length_gen.gen_value(context)? as usize;

        let mut i = 0;

        while i < num_iterations {
            let mut buf = [0u8; 1024];
            let mut buf_end = 0;

            while i < num_iterations && buf_end < (buf.len() - 6) {
                let c = self.gen_char(context)?;
                let slice = c.encode_utf8(&mut buf[buf_end..]);
                i += slice.len();
                buf_end += slice.len();
            }
            let as_str = unsafe { ::std::str::from_utf8_unchecked(&buf[..buf_end]) };
            out.write_str(as_str)?;
        }

        Ok(())
    }
}

fn create_string_gen(args: Arguments) -> CreateFunctionResult {
    let (length_gen, min_cp_inclusive, max_cp_inclusive) = args.require_3_args(
        "length",
        AnyFunction::require_uint,
        "min_codepoint_inclusive",
        AnyFunction::require_uint,
        "max_codepoint_inclusive",
        AnyFunction::require_uint,
    )?;
    Ok(AnyFunction::String(Rc::new(StringGenerator {
        length_gen,
        max_cp_inclusive,
        min_cp_inclusive,
    })))
}

pub const STRING_GEN_BUILTIN: &BuiltinFunctionPrototype = &BuiltinFunctionPrototype {
    function_name: "string",
    description:
        "constructs a string using the given length and min and max code point values inclusive",
    arguments: &[
        ("length", GenType::Uint),
        ("min_codepoint_inclusive", GenType::Uint),
        ("max_codepoint_inclusive", GenType::Uint),
    ],
    variadic: false,
    create_fn: &create_string_gen,
};

#[derive(Debug)]
struct StringLength {
    wrapped: DynStringFun,
}

impl RunnableFunction<u64> for StringLength {
    fn gen_value(&self, ctx: &mut ProgramContext) -> Result<u64, Error> {
        self.wrapped.gen_value(ctx).map(|value| value.len() as u64)
    }
    fn write_value(&self, ctx: &mut ProgramContext, out: &mut DataGenOutput) -> Result<(), Error> {
        let len = self.wrapped.gen_value(ctx)?.len() as u64;
        out.write(&len)
    }
}

fn create_str_len(args: Arguments) -> CreateFunctionResult {
    let wrapped = args.required_arg("string", 0, AnyFunction::require_string)?;
    Ok(AnyFunction::Uint(Rc::new(StringLength { wrapped })))
}

pub const STRING_LENGTH_BUILTIN: &BuiltinFunctionPrototype = &BuiltinFunctionPrototype {
    function_name: "string_length",
    description: "returns the length in utf8-encoded bytes of the generated string",
    arguments: &[("string", GenType::String)],
    variadic: false,
    create_fn: &create_str_len,
};

#[derive(Debug)]
struct StringBytes {
    encoding: DynStringFun,
    string: DynStringFun,
}

impl RunnableFunction<Vec<u8>> for StringBytes {
    fn gen_value(&self, context: &mut ProgramContext) -> Result<Vec<u8>, Error> {
        let encoding_label = self.encoding.gen_value(context)?;
        if let Some(encoding) = encoding_from_whatwg_label(&*encoding_label) {
            let value = self.string.gen_value(context)?;
            encoding.encode(&*value, EncoderTrap::Strict).map_err(|e| {
                format_err!(
                    "Failed to encode the input using encoding: {} -- error: {}",
                    encoding_label,
                    e
                )
            })
        } else {
            Err(format_err!("Invalid encoding label: '{}', encodings must be specified as a WHATWG label. See: https://encoding.spec.whatwg.org/#concept-encoding-get for more info", &*encoding_label))
        }
    }
    fn write_value(
        &self,
        context: &mut ProgramContext,
        out: &mut DataGenOutput,
    ) -> Result<(), Error> {
        let encoding_label = self.encoding.gen_value(context)?;
        if let Some(encoding) = encoding_from_whatwg_label(&*encoding_label) {
            let value = self.string.gen_value(context)?;
            out.with(move |output| {
                encoding
                    .encode_to(&*value, EncoderTrap::Strict, output)
                    .map_err(|e| {
                        format_err!(
                            "Failed to encode the input using encoding: {} -- error: {}",
                            encoding_label,
                            e
                        )
                    })
            })
        } else {
            Err(format_err!("Invalid encoding label: '{}', encodings must be specified as a WHATWG label. See: https://encoding.spec.whatwg.org/#concept-encoding-get for more info", &*encoding_label))
        }
    }
}

fn create_string_bytes(args: Arguments) -> CreateFunctionResult {
    let (encoding, string) = args.require_2_args(
        "encoding",
        AnyFunction::require_string,
        "string",
        AnyFunction::require_string,
    )?;

    Ok(AnyFunction::Bin(Rc::new(StringBytes { encoding, string })))
}

pub const STRING_ENCODE_BUILTIN: &BuiltinFunctionPrototype = &BuiltinFunctionPrototype {
    function_name: "string_bytes",
    description: "encodes strings using the given encoding, provided as a WHATWG encoding label",
    arguments: &[("encoding", GenType::String), ("string", GenType::String)],
    variadic: false,
    create_fn: &create_string_bytes,
};

#[cfg(test)]
mod test {
    use crate::fun_test::{assert_bin_output_is_expected, run_program, test_program_success};

    #[test]
    fn utf8_bytes_returns_encoded_string() {
        let program = r#"utf8_bytes("foo")"#;
        let expected = &[0x66, 0x6F, 0x6F];
        assert_bin_output_is_expected(program, expected);
    }

    #[test]
    fn string_bytes_supports_a_lot_of_encodings() {
        let program = r##"
        # This is not a 100% complete list of supported encodings, but it ought to cover most of what folks 
        # are likely to use.
        def encodings() = select(
            "unicode-1-1-utf-8",
            "utf-8",
            "utf8" ,
            "866",
            "cp866",
            "csibm866",
            "ibm866" ,
            "csisolatin2",
            "iso-8859-2",
            "iso-ir-101",
            "iso8859-2",
            "iso88592",
            "iso_8859-2",
            "iso_8859-2:1987",
            "l2",
            "latin2" ,
            "csisolatin3",
            "iso-8859-3",
            "iso-ir-109",
            "iso8859-3",
            "iso88593",
            "iso_8859-3",
            "iso_8859-3:1988",
            "l3",
            "latin3" ,
            "csisolatin4",
            "iso-8859-4",
            "iso-ir-110",
            "iso8859-4",
            "iso88594",
            "iso_8859-4",
            "iso_8859-4:1988",
            "l4",
            "latin4" ,
            "csisolatincyrillic",
            "cyrillic",
            "iso-8859-5",
            "iso-ir-144",
            "iso8859-5",
            "iso88595",
            "iso_8859-5",
            "iso_8859-5:1988" ,
            "arabic",
            "asmo-708",
            "csiso88596e",
            "csiso88596i",
            "csisolatinarabic",
            "ecma-114",
            "iso-8859-6",
            "iso-8859-6-e",
            "iso-8859-6-i",
            "iso-ir-127",
            "iso8859-6",
            "iso88596",
            "iso_8859-6",
            "iso_8859-6:1987" ,
            "ansi_x3.4-1968",
            "ascii",
            "cp1252",
            "cp819",
            "csisolatin1",
            "ibm819",
            "iso-8859-1",
            "iso-ir-100",
            "iso8859-1",
            "iso88591",
            "iso_8859-1",
            "iso_8859-1:1987",
            "l1",
            "latin1",
            "us-ascii",
            "windows-1252",
            "x-cp1252" ,
            "utf-16be",
            "utf-16",
            "utf-16le"
        );

        string_bytes(encodings(), "foo")
        "##;

        run_program(900, program).expect("Failed to run program");
    }

    #[test]
    fn std_unicode_string_fun() {
        let expected_output = "ༀʡㅱ⻇\u{1714}ⅼפּ𐌷﹗☔\u{243e}ᜇᜈ⤯뻳徻အ𐌇\u{c00}ⓕ\u{3101}⟪Ⓔℒலጴ꒽⧶◯Ѣㅉ \u{242a}☖㎲ඍ∯೦₮\u{20d3}\u{6e4}⽝\u{eb4}⽭ඊㇹ㈤ゼ⡀Ǜԇ𐎒𐎍\u{efd1}\u{fe07}ᙶ﹁き﹛\u{196e}\u{e64}෴Ɯꏕ\u{7a7}ᚇ⫞⟏𝀀ਜ਼ΠᏳᥜჴស႔㩎\u{af4}ὀァ㏔◉むዤβ⟙\u{ec8}ᢟ챞㍇⁽क\u{1007b}\u{fe09}ꇛɔ᧸ꂩ෨﹊";
        let input = r#"unicode_chars(100)"#;
        test_program_success(1, input, expected_output);
    }

    #[test]
    fn string_length_returns_length_of_string() {
        let program = r#"string_length(ascii_alphanumeric_chars(13))"#;
        let expected = "13";
        test_program_success(1, program, expected);
    }

    #[test]
    fn generate_ascii_strings() {
        let expected_output = "a6OqR822C3hoTTf1";
        let input = "ascii_alphanumeric_chars(uint(0, 10))";
        test_program_success(4, input, expected_output);
    }

}

use failure::Error;
use std::rc::Rc;
use ::{
    AnyFunction, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput, DynCharFun,
    DynUintFun, DynStringFun, FunctionPrototype, GenType, ProgramContext, RunnableFunction, Arguments,
};
use IString;

#[derive(Debug)]
pub struct StringGenerator {
    length_gen: DynUintFun,
    char_gen: DynCharFun,
}

impl RunnableFunction<IString> for StringGenerator {
    fn gen_value(&self, context: &mut ProgramContext) -> Result<IString, Error> {
        let len = self.length_gen.gen_value(context)?;
        let mut buf = String::with_capacity(len as usize);

        for _ in 0..len {
            let character = self.char_gen.gen_value(context)?;
            buf.push(character);
        }
        Ok(buf.into())
    }

    fn write_value(
        &self,
        context: &mut ProgramContext,
        out: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        let len = self.length_gen.gen_value(context)?;
        let mut total = 0;
        for _ in 0..len {
            total += self.char_gen.write_value(context, out)?;
        }
        Ok(total)
    }
}

fn create_string_gen(args: Arguments) -> CreateFunctionResult {
    let (length, chars) = args.require_2_args(
        "length",
        AnyFunction::require_uint,
        "characters",
        AnyFunction::require_char,
    )?;

    Ok(AnyFunction::String(Rc::new(StringGenerator {
        length_gen: length,
        char_gen: chars,
    })))
}

pub const STRING_GEN_BUILTIN: &FunctionPrototype =
    &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
        function_name: "string",
        description: "constructs a string using the given length and character generators",
        arguments: &[("length", GenType::Uint), ("characters", GenType::Char)],
        variadic: false,
        create_fn: &create_string_gen,
    });


#[derive(Debug)]
struct StringLength {
    wrapped: DynStringFun,
}

impl RunnableFunction<u64> for StringLength {
    fn gen_value(&self, ctx: &mut ProgramContext) -> Result<u64, Error> {
        self.wrapped.gen_value(ctx).map(|value| {
            value.len() as u64
        })
    }
    fn write_value(&self, ctx: &mut ProgramContext, out: &mut DataGenOutput) -> Result<u64, Error> {
        let len = self.wrapped.gen_value(ctx)?.len() as u64;
        out.write(&len)
    }
}

fn create_str_len(args: Arguments) -> CreateFunctionResult {
    let wrapped = args.required_arg("string", 0, AnyFunction::require_string)?;
    Ok(AnyFunction::Uint(Rc::new(StringLength {
        wrapped
    })))
}

pub const STRING_LENGTH_BUILTIN: &FunctionPrototype =
    &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
        function_name: "string_length",
        description: "returns the length in utf8-encoded bytes of the generated string",
        arguments: &[("string", GenType::String)],
        variadic: false,
        create_fn: &create_str_len,
    });


#[cfg(test)]
mod test {
    use fun_test::test_program_success;


    #[test]
    fn std_unicode_string_fun() {
        let expected_output = "ༀʠㅰ⻆\u{1713}ⅻףּ𐌷﹗☔\u{243d}ᜇᜑᥤ慧\u{df1}ખ@䷪ǲ𐌚ℍṄﵗ㎕𐑩︲ᵕቢ\u{2429}☖㎱ඍ∯೦₮\u{20ed}⁺⁅␌ㇹ㈤ゼ⡀Ǜԇ𐎑𐎍\u{efd0}≈𥴘き﹛ᥚ數𐎁য়ﺂᚇ⫝⟏𝀀ਗ਼ΠᏲᥜჳស႔㠛𐎌პś﹀₾ㅉ⨏⇍☹\u{1885}擎⁽୮\u{fe09}ꀬˬꂩ෨﹊Ⓗភڊ깞ᜇ੯⻃\u{fe28}\u{a55}ԟ⼜";
        let input = r#"unicode_string(100)"#;
        test_program_success(1, input, expected_output);
    }

    #[test]
    fn string_length_returns_length_of_string() {
        let program = r#"string_length(alphanumeric_string(13))"#;
        let expected = "13";
        test_program_success(1, program, expected);
    }

    #[test]
    fn generate_ascii_strings() {
        let expected_output = "w6U9vomgJ4gxen0XO";
        let input = "alphanumeric_string(uint(0, 10))";
        test_program_success(4, input, expected_output);
    }

    #[test]
    fn use_custom_string_function() {
        let input = r#"
            def consonants() = select('b', 'c', 'd', 'f', 'g', 'h', 'j', 'k', 
                    'l', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z');
            def vowels() = select('a', 'e', 'i', 'o', 'u');

            def chars() = select(vowels(), consonants());

            string(10, chars())
        "#;
        let expected_output = "ausjmhaevg";
        test_program_success(1, input, expected_output);
    }
}
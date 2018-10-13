use failure::Error;
use std::rc::Rc;
use v2::{
    AnyFunction, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput, DynCharFun,
    DynUintFun, FunctionPrototype, GenType, ProgramContext, RunnableFunction, Arguments,
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

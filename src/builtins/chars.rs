use failure::Error;
use std::rc::Rc;
use {
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput,
    DynUintFun, GenType, ProgramContext, RunnableFunction,
};

#[derive(Debug)]
struct CharGenerator {
    min_inclusive: DynUintFun,
    max_inclusive: DynUintFun,
}

impl RunnableFunction<char> for CharGenerator {
    fn gen_value(&self, context: &mut ProgramContext) -> Result<char, Error> {
        let min = self.min_inclusive.gen_value(context)?;
        let max = self.max_inclusive.gen_value(context)?;

        let as_u64 = context.gen_range_inclusive(min, max);

        ::std::char::from_u32(as_u64 as u32).ok_or_else(|| {
            format_err!("Invalid unicode codepoint: {}, generated from range: min_inclusive: {}, max_inclusive: {}", as_u64, min, max)
        })
    }

    fn write_value(
        &self,
        context: &mut ProgramContext,
        output: &mut DataGenOutput,
    ) -> Result<(), Error> {
        let value = self.gen_value(context)?;
        output.write(&value)
    }
}

const MIN_ARG: &str = "min_inclusive";
const MAX_ARG: &str = "max_inclusive";

fn create_char_gen(args: Arguments) -> CreateFunctionResult {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let (min, max) = args.require_2_args(
        MIN_ARG, AnyFunction::require_uint,
        MAX_ARG, AnyFunction::require_uint,
    )?;

    Ok(AnyFunction::Char(Rc::new(CharGenerator {
        min_inclusive: min,
        max_inclusive: max,
    })))
}

pub const CHAR_GEN_BUILTIN: &BuiltinFunctionPrototype = &BuiltinFunctionPrototype {
    function_name: "char",
    description:
        "selects a single random character from within the provided range of unicode codepoints",
    arguments: &[(MIN_ARG, GenType::Uint), (MAX_ARG, GenType::Uint)],
    variadic: false,
    create_fn: &create_char_gen,
};

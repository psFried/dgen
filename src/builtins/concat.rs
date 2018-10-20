use failure::Error;
use std::rc::Rc;
use {
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput,
    DynBinFun, DynStringFun, FunctionPrototype, GenType, IString, ProgramContext, RunnableFunction,
};

#[derive(Debug)]
struct Concat<T> {
    funs: Vec<T>,
}

impl RunnableFunction<IString> for Concat<DynStringFun> {
    fn gen_value(&self, context: &mut ProgramContext) -> Result<IString, Error> {
        let mut buffer = Vec::new();
        {
            let mut out = DataGenOutput::new(&mut buffer);
            for fun in self.funs.iter() {
                fun.write_value(context, &mut out)?;
            }
        }
        // should be ok here since we know the string functions are generating valid utf8
        let string = unsafe { String::from_utf8_unchecked(buffer) };
        Ok(string.into())
    }

    fn write_value(
        &self,
        context: &mut ProgramContext,
        out: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        let mut total = 0;
        for fun in self.funs.iter() {
            total += fun.write_value(context, out)?;
        }
        Ok(total)
    }
}

impl RunnableFunction<Vec<u8>> for Concat<DynBinFun> {
    fn gen_value(&self, ctx: &mut ProgramContext) -> Result<Vec<u8>, Error> {
        let mut buffer = self.funs[0].gen_value(ctx)?;
        if self.funs.len() > 1 {
            for fun in self.funs.iter().skip(1) {
                let result = fun.gen_value(ctx)?;
                buffer.extend_from_slice(&result[..]);
            }
        }
        Ok(buffer)
    }

    fn write_value(&self, ctx: &mut ProgramContext, out: &mut DataGenOutput) -> Result<u64, Error> {
        let mut total = 0;
        for fun in self.funs.iter() {
            total += fun.write_value(ctx, out)?;
        }
        Ok(total)
    }
}

const CONCAT_ARG_NAME: &str = "string_value";

fn create_concat(args: Arguments) -> CreateFunctionResult {
    let funs = args.get_required_varargs(CONCAT_ARG_NAME, 0, AnyFunction::require_string)?;
    Ok(AnyFunction::String(Rc::new(Concat { funs })))
}

fn create_concat_bin(args: Arguments) -> CreateFunctionResult {
    let funs = args.get_required_varargs(CONCAT_ARG_NAME, 0, AnyFunction::require_bin)?;
    Ok(AnyFunction::Bin(Rc::new(Concat { funs })))
}
pub const CONCAT_BUILTIN: &FunctionPrototype =
    &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
        function_name: "concat",
        description: "concatenates the input strings into a single output string",
        arguments: &[(CONCAT_ARG_NAME, GenType::String)],
        variadic: true,
        create_fn: &create_concat,
    });

pub const CONCAT_BIN_BUILTIN: &FunctionPrototype =
    &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
        function_name: "concat",
        description: "concatenates the input bytes into a single output",
        arguments: &[(CONCAT_ARG_NAME, GenType::Bin)],
        variadic: true,
        create_fn: &create_concat_bin,
    });

#[cfg(test)]
mod test {
    use fun_test::{assert_bin_output_is_expected, test_program_success};

    #[test]
    fn binary_is_concatenated() {
        let program = r##"
        concat([0x05], [], [0x01, 0xff])
        "##;
        let expected = &[0x05, 0x01, 0xff];
        assert_bin_output_is_expected(program, expected);
    }

    #[test]
    fn strings_are_concatenated() {
        let program = r##"
        concat("foo", "", "bar", "baz")
        "##;
        let expected = "foobarbaz";
        test_program_success(1, program, expected);
    }
}

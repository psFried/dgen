use failure::Error;
use std::rc::Rc;
use ::{
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput,
    DynStringFun, FunctionPrototype, GenType, ProgramContext, RunnableFunction, IString,
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

const CONCAT_ARG_NAME: &str = "string_value";

fn create_concat(args: Arguments) -> CreateFunctionResult {
    let funs = args.get_required_varargs(CONCAT_ARG_NAME, 0, AnyFunction::require_string)?;
    Ok(AnyFunction::String(Rc::new(Concat { funs })))
}

pub const CONCAT_BUILTIN: &FunctionPrototype =
    &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
        function_name: "concat",
        description: "concatenates the input strings into a single output string",
        arguments: &[(CONCAT_ARG_NAME, GenType::String)],
        variadic: true,
        create_fn: &create_concat,
    });

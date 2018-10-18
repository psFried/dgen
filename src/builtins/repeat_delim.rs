use failure::Error;
use std::rc::Rc;
use ::{
    AnyFunction, BuiltinFunctionPrototype, FunctionPrototype, CreateFunctionResult, DataGenOutput, DynUintFun,
    ProgramContext, RunnableFunction, GenType, Arguments, DynFun
};
use ::IString;


#[derive(Debug)]
struct RepeatDelimited<T> {
    count: DynUintFun,
    prefix: DynFun<T>,
    delimiter: DynFun<T>,
    repeated: DynFun<T>,
    suffix: DynFun<T>,
}

impl RunnableFunction<IString> for RepeatDelimited<IString> {
    fn gen_value(&self, ctx: &mut ProgramContext) -> Result<IString, Error> {
        let mut buffer = Vec::new();
        {
            let mut out = DataGenOutput::new(&mut buffer);
            self.write_value(ctx, &mut out)?;
        }

        let str_val = unsafe { String::from_utf8_unchecked(buffer) };
        Ok(str_val.into())
    }

    fn write_value(&self, ctx: &mut ProgramContext, out: &mut DataGenOutput) -> Result<u64, Error> {
        let count = self.count.gen_value(ctx)?;
        let mut total = 0;
        self.prefix.write_value(ctx, out)?;

        for i in 0..count {
            if i > 0 {
                total += self.delimiter.write_value(ctx, out)?;
            }
            total += self.repeated.write_value(ctx, out)?;
        }
        total += self.suffix.write_value(ctx, out)?;
        Ok(total)
    }
}

const COUNT_PARAM: &str = "count";
const PREFIX_PARAM: &str = "prefix";
const TO_REPEAT_PARAM: &str = "to_repeat";
const DELIMITER_PARAM: &str = "delimiter";
const SUFFIX_PARAM: &str = "suffix";

fn create_repeat_delim(args: Arguments) -> CreateFunctionResult {
    let count = args.required_arg(COUNT_PARAM, 0, AnyFunction::require_uint)?;
    let prefix = args.required_arg(PREFIX_PARAM, 1, AnyFunction::require_string)?;
    let repeated = args.required_arg(TO_REPEAT_PARAM, 2, AnyFunction::require_string)?;
    let delimiter = args.required_arg(DELIMITER_PARAM, 3, AnyFunction::require_string)?;
    let suffix = args.required_arg(SUFFIX_PARAM, 4, AnyFunction::require_string)?;

    let fun = RepeatDelimited {
        count,
        prefix,
        repeated,
        delimiter,
        suffix,
    };
    Ok(AnyFunction::String(Rc::new(fun)))
}

pub const REPEAT_DELIM_BUILTIN: &FunctionPrototype = &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
    function_name: "repeat_delimited",
    description: "Formats the output by repeating the given generator separated by the delimiter",
    arguments: &[
        (COUNT_PARAM, GenType::Uint),
        (PREFIX_PARAM, GenType::String),
        (TO_REPEAT_PARAM, GenType::String),
        (DELIMITER_PARAM, GenType::String),
        (SUFFIX_PARAM, GenType::String)
    ],
    variadic: false,
    create_fn: &create_repeat_delim,
});




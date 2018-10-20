use failure::Error;
use std::rc::Rc;
use IString;
use {
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput, DynFun,
    DynUintFun, FunctionPrototype, GenType, ProgramContext, RunnableFunction,
};

#[derive(Debug)]
struct RepeatDelimited<T> {
    count: DynUintFun,
    prefix: DynFun<T>,
    delimiter: DynFun<T>,
    repeated: DynFun<T>,
    suffix: DynFun<T>,
}

impl<T> RepeatDelimited<T> {
    fn do_write(&self, ctx: &mut ProgramContext, out: &mut DataGenOutput) -> Result<u64, Error> {
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
        self.do_write(ctx, out)
    }
}

impl RunnableFunction<Vec<u8>> for RepeatDelimited<Vec<u8>> {
    fn gen_value(&self, ctx: &mut ProgramContext) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::with_capacity(64);
        {
            self.write_value(ctx, &mut DataGenOutput::new(&mut buffer))?;
        }
        Ok(buffer)
    }

    fn write_value(&self, ctx: &mut ProgramContext, out: &mut DataGenOutput) -> Result<u64, Error> {
        self.do_write(ctx, out)
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
fn create_bin_repeat_delim(args: Arguments) -> CreateFunctionResult {
    let count = args.required_arg(COUNT_PARAM, 0, AnyFunction::require_uint)?;
    let prefix = args.required_arg(PREFIX_PARAM, 1, AnyFunction::require_bin)?;
    let repeated = args.required_arg(TO_REPEAT_PARAM, 2, AnyFunction::require_bin)?;
    let delimiter = args.required_arg(DELIMITER_PARAM, 3, AnyFunction::require_bin)?;
    let suffix = args.required_arg(SUFFIX_PARAM, 4, AnyFunction::require_bin)?;

    let fun = RepeatDelimited {
        count,
        prefix,
        repeated,
        delimiter,
        suffix,
    };
    Ok(AnyFunction::Bin(Rc::new(fun)))
}

pub const REPEAT_DELIM_BUILTIN: &FunctionPrototype =
    &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
        function_name: "repeat_delimited",
        description:
            "Formats the output by repeating the given generator separated by the delimiter",
        arguments: &[
            (COUNT_PARAM, GenType::Uint),
            (PREFIX_PARAM, GenType::String),
            (TO_REPEAT_PARAM, GenType::String),
            (DELIMITER_PARAM, GenType::String),
            (SUFFIX_PARAM, GenType::String),
        ],
        variadic: false,
        create_fn: &create_repeat_delim,
    });

pub const REPEAT_DELIM_BIN_BUILTIN: &FunctionPrototype =
    &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
        function_name: "repeat_delimited",
        description:
            "Formats the output by repeating the given generator separated by the delimiter",
        arguments: &[
            (COUNT_PARAM, GenType::Uint),
            (PREFIX_PARAM, GenType::Bin),
            (TO_REPEAT_PARAM, GenType::Bin),
            (DELIMITER_PARAM, GenType::Bin),
            (SUFFIX_PARAM, GenType::Bin),
        ],
        variadic: false,
        create_fn: &create_bin_repeat_delim,
    });

#[cfg(test)]
mod test {
    use fun_test::{assert_bin_output_is_expected, test_program_success};

    #[test]
    fn repeat_delimited_creates_string_output() {
        let program = r##"repeat_delimited(3, "start: ", "R", ", ", " :end") "##;
        let expected = "start: R, R, R :end";
        test_program_success(1, program, expected);
    }

    #[test]
    fn repeat_delimited_creates_binary_output() {
        let program = r#"repeat_delimited(3, [0x01, 0x02], [0xFF], [0xAA, 0xAF], [0x06])"#;
        let expected = &[0x01, 0x02, 0xff, 0xaa, 0xaf, 0xff, 0xaa, 0xaf, 0xff, 0x06];
        assert_bin_output_is_expected(program, expected);
    }

}

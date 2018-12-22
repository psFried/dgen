use failure::Error;
use std::fmt::Debug;
use std::rc::Rc;
use {
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput, DynFun,
    GenType, OutputType, ProgramContext, RunnableFunction,
};

use rand::distributions::uniform::SampleUniform;

#[derive(Debug)]
struct NumericGen<T> {
    min_inclusive: DynFun<T>,
    max_inclusive: DynFun<T>,
}

impl<T: Debug + PartialOrd + Copy + SampleUniform + OutputType> RunnableFunction<T>
    for NumericGen<T>
{
    fn gen_value(&self, ctx: &mut ProgramContext) -> Result<T, Error> {
        let min = self.min_inclusive.gen_value(ctx)?;
        let max = self.max_inclusive.gen_value(ctx)?;
        let result = ctx.gen_range_inclusive(min, max);
        Ok(result)
    }

    fn write_value(&self, ctx: &mut ProgramContext, out: &mut DataGenOutput) -> Result<(), Error> {
        let value = self.gen_value(ctx)?;
        out.write(&value)
    }
}

const MIN_PARAM: &str = "min_inclusive";
const MAX_PARAM: &str = "max_inclusive";

macro_rules! make_numeric_builtin {
    ($proto_name:ident, $create_fn_name:ident, $gen_type:expr, $any_fun_path:path, $convert_fun:path, $fun_name:expr) => {

        fn $create_fn_name(args: Arguments) -> CreateFunctionResult {
            let (min_inclusive, max_inclusive) = args.require_2_args(MIN_PARAM, $convert_fun, MAX_PARAM, $convert_fun)?;
            let fun = NumericGen {
                min_inclusive,
                max_inclusive
            };
            Ok($any_fun_path(Rc::new(fun)))
        }

        pub const $proto_name: &BuiltinFunctionPrototype = &BuiltinFunctionPrototype {
            function_name: $fun_name,
            description: "Generates a number between the given given minimum (inclusive) and maximum (exclusive)",
            arguments: &[
                (MIN_PARAM, $gen_type),
                (MAX_PARAM, $gen_type),
            ],
            variadic: false,
            create_fn: &$create_fn_name,
        };

    };
}

make_numeric_builtin!(
    UINT_BUILTIN,
    create_uint_builtin,
    GenType::Uint,
    AnyFunction::Uint,
    AnyFunction::require_uint,
    "uint"
);
make_numeric_builtin!(
    INT_BUILTIN,
    create_int_builtin,
    GenType::Int,
    AnyFunction::Int,
    AnyFunction::require_int,
    "int"
);
make_numeric_builtin!(
    DECIMAL_BUILTIN,
    create_decimal_builtin,
    GenType::Decimal,
    AnyFunction::Decimal,
    AnyFunction::require_decimal,
    "decimal"
);

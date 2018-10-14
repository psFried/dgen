use failure::Error;
use std::fmt::Debug;
use std::rc::Rc;
use v2::{
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput, DynFun,
    FunctionPrototype, GenType, ProgramContext, RunnableFunction,
};

#[derive(Debug)]
struct SelectFun<T> {
    wrapped: Vec<DynFun<T>>,
}

fn select_fun<'a, 'b, T>(ctx: &'a mut ProgramContext, values: &'b [DynFun<T>]) -> &'b DynFun<T> {
    let i = ctx.gen_range(0, values.len());
    &values[i]
}

impl<T: Debug> RunnableFunction<T> for SelectFun<T> {
    fn gen_value(&self, ctx: &mut ProgramContext) -> Result<T, Error> {
        let fun = select_fun(ctx, self.wrapped.as_slice());
        fun.gen_value(ctx)
    }
    fn write_value(&self, ctx: &mut ProgramContext, out: &mut DataGenOutput) -> Result<u64, Error> {
        let fun = select_fun(ctx, self.wrapped.as_slice());
        fun.write_value(ctx, out)
    }
}

const SELECT_ARG: &str = "gen";

macro_rules! make_select_proto {
    ($name:ident, $create_fn_name:ident, $gen_type:expr, $any_fun_type:path, $ret_type:ty, $convert_fun:path) => {
        fn $create_fn_name(args: Arguments) -> CreateFunctionResult {
            let as_types = args.get_required_varargs(SELECT_ARG, 0, $convert_fun)?;
            let sel = Rc::new(SelectFun { wrapped: as_types });
            let any = $any_fun_type(sel);
            Ok(any)
        }

        pub const $name: &FunctionPrototype =
            &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
                function_name: "select",
                description: "Randomly selects one of the input functions",
                arguments: &[(SELECT_ARG, $gen_type)],
                variadic: true,
                create_fn: &$create_fn_name,
            });
    };
}

make_select_proto!(
    SELECT_CHAR_BUILTIN,
    create_select_char,
    GenType::Char,
    AnyFunction::Char,
    char,
    AnyFunction::require_char
);

make_select_proto!(
    SELECT_STRING_BUILTIN,
    create_select_string,
    GenType::String,
    AnyFunction::String,
    IString,
    AnyFunction::require_string
);

make_select_proto!(
    SELECT_BOOLEAN_BUILTIN,
    create_select_boolean,
    GenType::Boolean,
    AnyFunction::Boolean,
    bool,
    AnyFunction::require_boolean
);

make_select_proto!(
    SELECT_UINT_BUILTIN,
    create_select_uint,
    GenType::Uint,
    AnyFunction::Uint,
    u64,
    AnyFunction::require_uint
);

make_select_proto!(
    SELECT_INT_BUILTIN,
    create_select_int,
    GenType::Int,
    AnyFunction::Int,
    i64,
    AnyFunction::require_int
);

make_select_proto!(
    SELECT_DECIMAL_BUILTIN,
    create_select_decimal,
    GenType::Decimal,
    AnyFunction::Decimal,
    f64,
    AnyFunction::require_decimal
);

make_select_proto!(
    SELECT_BIN_BUILTIN,
    create_select_bin,
    GenType::Bin,
    AnyFunction::Bin,
    Vec<u8>,
    AnyFunction::require_bin
);

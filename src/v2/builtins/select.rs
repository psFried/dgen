use failure::Error;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use v2::{
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput, DynFun,
    FunctionPrototype, GenType, ProgramContext, RunnableFunction,
};

#[derive(Debug)]
struct StableSelectFun<T> {
    wrapped: Vec<DynFun<T>>,
    index: RefCell<Option<usize>>,
}

impl<T> StableSelectFun<T> {
    fn get_function(&self, ctx: &mut ProgramContext) -> DynFun<T> {
        let StableSelectFun {
            ref wrapped,
            ref index,
        } = *self;
        let mut index = index.borrow_mut();
        if index.is_none() {
            *index = Some(ctx.gen_range(0, wrapped.len()));
        }

        wrapped[index.unwrap()].clone()
    }
}

impl<T: Debug> RunnableFunction<T> for StableSelectFun<T> {
    fn gen_value(&self, ctx: &mut ProgramContext) -> Result<T, Error> {
        let fun = self.get_function(ctx);
        fun.gen_value(ctx)
    }
    fn write_value(&self, ctx: &mut ProgramContext, out: &mut DataGenOutput) -> Result<u64, Error> {
        let fun = self.get_function(ctx);
        fun.write_value(ctx, out)
    }
}

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
    ($select_name:ident, $create_select_fn_name:ident, $stable_select_name:ident, $create_stable_select_fn_name:ident, $gen_type:expr, $any_fun_type:path, $ret_type:ty, $convert_fun:path) => {

        fn $create_select_fn_name(args: Arguments) -> CreateFunctionResult {
            let as_types = args.get_required_varargs(SELECT_ARG, 0, $convert_fun)?;
            let sel = Rc::new(SelectFun { wrapped: as_types });
            let any = $any_fun_type(sel);
            Ok(any)
        }

        pub const $select_name: &FunctionPrototype =
            &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
                function_name: "select",
                description: "Randomly selects one of the input functions",
                arguments: &[(SELECT_ARG, $gen_type)],
                variadic: true,
                create_fn: &$create_select_fn_name,
            });

        fn $create_stable_select_fn_name(args: Arguments) -> CreateFunctionResult {
            let as_types = args.get_required_varargs(SELECT_ARG, 0, $convert_fun)?;
            let sel = Rc::new(StableSelectFun {
                wrapped: as_types,
                index: RefCell::new(None),
            });
            let any = $any_fun_type(sel);
            Ok(any)
        }

        pub const $stable_select_name: &FunctionPrototype =
            &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
                function_name: "stable_select",
                description: "Randomly selects one of the input functions and continues to select that same function forever",
                arguments: &[(SELECT_ARG, $gen_type)],
                variadic: true,
                create_fn: &$create_stable_select_fn_name,
            });
    };
}

make_select_proto!(
    SELECT_CHAR_BUILTIN,
    create_select_char,
    STABLE_SELECT_CHAR_BUILTIN,
    create_stable_select_char,
    GenType::Char,
    AnyFunction::Char,
    char,
    AnyFunction::require_char
);

make_select_proto!(
    SELECT_STRING_BUILTIN,
    create_select_string,
    STABLE_SELECT_STRING_BUILTIN,
    create_stable_select_string,
    GenType::String,
    AnyFunction::String,
    IString,
    AnyFunction::require_string
);

make_select_proto!(
    SELECT_BOOLEAN_BUILTIN,
    create_select_boolean,
    STABLE_SELECT_BOOLEAN_BUILTIN,
    create_stable_select_boolean,
    GenType::Boolean,
    AnyFunction::Boolean,
    bool,
    AnyFunction::require_boolean
);

make_select_proto!(
    SELECT_UINT_BUILTIN,
    create_select_uint,
    STABLE_SELECT_UINT_BUILTIN,
    create_stable_select_uint,
    GenType::Uint,
    AnyFunction::Uint,
    u64,
    AnyFunction::require_uint
);

make_select_proto!(
    SELECT_INT_BUILTIN,
    create_select_int,
    STABLE_SELECT_INT_BUILTIN,
    create_stable_select_int,
    GenType::Int,
    AnyFunction::Int,
    i64,
    AnyFunction::require_int
);

make_select_proto!(
    SELECT_DECIMAL_BUILTIN,
    create_select_decimal,
    STABLE_SELECT_DECIMAL_BUILTIN,
    create_stable_select_decimal,
    GenType::Decimal,
    AnyFunction::Decimal,
    f64,
    AnyFunction::require_decimal
);

make_select_proto!(
    SELECT_BIN_BUILTIN,
    create_select_bin,
    STABLE_SELECT_BIN_BUILTIN,
    create_stable_select_bin,
    GenType::Bin,
    AnyFunction::Bin,
    Vec<u8>,
    AnyFunction::require_bin
);

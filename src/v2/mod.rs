mod builtins;
mod context;
pub(crate) mod interpreter;
mod prototype;
mod types;

use std::fmt::Debug;
use std::rc::Rc;
use IString;

pub use self::context::ProgramContext;
pub use self::interpreter::Interpreter;
pub use self::prototype::{
    BoundArgument, BuiltinFunctionCreator, BuiltinFunctionPrototype, CreateFunctionResult,
    FunctionPrototype, InterpretedFunctionPrototype,
};
pub use self::types::{
    ConstBin, ConstBoolean, ConstChar, ConstDecimal, ConstInt, ConstString, ConstUint, GenType,
    OutputType,
};
pub use writer::DataGenOutput;

use failure::Error;

pub trait RunnableFunction<T>: Debug {
    fn gen_value(&self, context: &mut ProgramContext) -> Result<T, Error>;

    fn write_value(
        &self,
        context: &mut ProgramContext,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error>;
}

pub type DynFun<T> = Rc<RunnableFunction<T>>;

pub type DynStringFun = DynFun<IString>;
pub type DynCharFun = DynFun<char>;
pub type DynUintFun = DynFun<u64>;
pub type DynIntFun = DynFun<i64>;
pub type DynDecimalFun = DynFun<f64>;
pub type DynBooleanFun = DynFun<bool>;
pub type DynBinFun = DynFun<Vec<u8>>;

#[derive(Debug, Clone)]
pub enum AnyFunction {
    String(DynStringFun),
    Char(DynCharFun),
    Uint(DynUintFun),
    Int(DynIntFun),
    Decimal(DynDecimalFun),
    Boolean(DynBooleanFun),
    Bin(DynBinFun),
}

impl AnyFunction {
    pub fn get_type(&self) -> GenType {
        match *self {
            AnyFunction::String(_) => GenType::String,
            AnyFunction::Char(_) => GenType::Char,
            AnyFunction::Uint(_) => GenType::Uint,
            AnyFunction::Int(_) => GenType::Int,
            AnyFunction::Decimal(_) => GenType::Decimal,
            AnyFunction::Boolean(_) => GenType::Boolean,
            AnyFunction::Bin(_) => GenType::Bin,
        }
    }
}

macro_rules! type_conversions {
    ($([$as_fn_name:ident, $req_fn_name:ident, $return_type:ty, $do_match:path]),*) => {
        impl AnyFunction {

            $(
            pub fn $as_fn_name(self) -> Result<$return_type, AnyFunction> {
                match self {
                    $do_match(fun) => Ok(fun),
                    other @ _ => Err(other)
                }
            }

            pub fn $req_fn_name(self) -> Result<$return_type, Error> {
                self.$as_fn_name().map_err(|fun| {
                    format_err!("Invalid argument type, expected: {}, actual: {}", stringify!($return_type), fun.get_type())
                })
            }
            )*

        }
    }
}

type_conversions!{
    [as_string, require_string, DynStringFun, AnyFunction::String],
    [as_char, require_char, DynCharFun, AnyFunction::Char],
    [as_int, require_int, DynIntFun, AnyFunction::Int],
    [as_uint, require_uint, DynUintFun, AnyFunction::Uint],
    [as_decimal, require_decimal, DynDecimalFun, AnyFunction::Decimal],
    [as_boolean, require_boolean, DynBooleanFun, AnyFunction::Boolean],
    [as_bin, require_bin, DynBinFun, AnyFunction::Bin]
}

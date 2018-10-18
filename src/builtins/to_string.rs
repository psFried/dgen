use failure::Error;
use std::fmt::{Debug, Display};
use std::rc::Rc;
use ::{
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput, DynFun,
    FunctionPrototype, GenType, OutputType, ProgramContext, RunnableFunction,
};
use IString;

#[derive(Debug)]
struct ToString<T: Display + OutputType> {
    gen: DynFun<T>,
}

impl<T: Display + Debug + OutputType + 'static> ToString<T> {
    fn new(wrapped: DynFun<T>) -> AnyFunction {
        AnyFunction::String(Rc::new(ToString { gen: wrapped }))
    }
}

// Display bound is here to prevent this impl from being used to format binary
impl<T: Display + Debug + OutputType> RunnableFunction<IString> for ToString<T> {
    fn gen_value(&self, ctx: &mut ProgramContext) -> Result<IString, Error> {
        let mut buffer = Vec::with_capacity(32);
        {
            let mut out = DataGenOutput::new(&mut buffer);
            self.write_value(ctx, &mut out)?;
        }
        // we know that the result will be valid utf8 because the implementations of OutputType for non-binary
        // are all guaranteed to produce valid utf-8
        let as_str = unsafe { String::from_utf8_unchecked(buffer) };
        Ok(as_str.into())
    }
    fn write_value(&self, ctx: &mut ProgramContext, out: &mut DataGenOutput) -> Result<u64, Error> {
        // just write it out directly, since we know the output type is not binary
        self.gen.write_value(ctx, out)
    }
}

const TO_STRING_PARAM: &str = "fun";

fn create_to_string(args: Arguments) -> CreateFunctionResult {
    let fun = args.require_any(TO_STRING_PARAM, 0)?;

    match fun {
        str_fun @ AnyFunction::String(_) => Ok(str_fun),
        AnyFunction::Boolean(fun) => Ok(ToString::new(fun)),
        AnyFunction::Char(fun) => Ok(ToString::new(fun)),
        AnyFunction::Decimal(fun) => Ok(ToString::new(fun)),
        AnyFunction::Int(fun) => Ok(ToString::new(fun)),
        AnyFunction::Uint(fun) => Ok(ToString::new(fun)),
        AnyFunction::Bin(_) => {
            Err(format_err!("Invalid binary argument to to_string function"))
        }
    }
}

macro_rules! make_to_string {
    ($proto_name:ident, $gen_type:expr) => {

        pub const $proto_name: &FunctionPrototype = &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
            function_name: "to_string",
            description: "Converts its input to a string using the default formating",
            arguments: &[(TO_STRING_PARAM, $gen_type)],
            variadic: false,
            create_fn: &create_to_string,
        });
    };
}

make_to_string!(BOOLEAN_TO_STRING_BUILTIN, GenType::Boolean);
make_to_string!(CHAR_TO_STRING_BUILTIN, GenType::Char);
make_to_string!(DECIMAL_TO_STRING_BUILTIN, GenType::Decimal);
make_to_string!(INT_TO_STRING_BUILTIN, GenType::Int);
make_to_string!(UINT_TO_STRING_BUILTIN, GenType::Uint);

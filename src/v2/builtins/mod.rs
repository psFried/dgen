mod chars;

use failure::Error;
use v2::{AnyFunction, GenType, FunctionPrototype};
use v2::interpreter::Module;

pub const BUILTIN_FNS: &'static [&'static FunctionPrototype] = &[
    self::chars::CHAR_GEN_BUILTIN,
];

fn wrong_argument_type(function_name: &str, expected: GenType, actual: AnyFunction) -> Error {
    format_err!(
        "Invalid argument for function '{}', expected type: {}, actual type: {}",
        function_name,
        expected,
        actual.get_type()
    )
}

trait ArgumentResult {
    fn required_arg(&mut self, arg_name: &'static str) -> Result<AnyFunction, Error>;

    fn required_args2<F1, R1, F2, R2>(
        &mut self,
        arg1_name: &'static str,
        af1: F1,
        arg2_name: &'static str,
        af2: F2,
    ) -> Result<(R1, R2), Error>
    where
        F1: FnOnce(AnyFunction) -> Result<R1, Error>,
        F2: FnOnce(AnyFunction) -> Result<R2, Error>,
    {
        let r2 = self.required_arg(arg2_name).and_then(af2)?;
        let r1 = self.required_arg(arg1_name).and_then(af1)?;
        Ok((r1, r2))
    }

    fn required_args3<F1, R1, F2, R2, F3, R3>(
        &mut self,
        arg1_name: &'static str,
        af1: F1,
        arg2_name: &'static str,
        af2: F2,
        arg3_name: &'static str,
        af3: F3,
    ) -> Result<(R1, R2), Error>
    where
        F1: FnOnce(AnyFunction) -> Result<R1, Error>,
        F2: FnOnce(AnyFunction) -> Result<R2, Error>,
        F3: FnOnce(AnyFunction) -> Result<R3, Error>,
    {
        let r3 = self.required_arg(arg3_name).and_then(af3)?;
        let r2 = self.required_arg(arg2_name).and_then(af2)?;
        let r1 = self.required_arg(arg1_name).and_then(af1)?;
        Ok((r1, r2))
    }
}

impl ArgumentResult for Vec<AnyFunction> {
    fn required_arg(&mut self, arg_name: &'static str) -> Result<AnyFunction, Error> {
        self.pop().ok_or_else(|| {
            format_err!(
                "Expected an argument for '{}', but no argument was provided in that position",
                arg_name
            )
        })
    }
}

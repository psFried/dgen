use failure::Error;
use std::rc::Rc;
use crate::{
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput,
    DynBinFun, GenType, ProgramContext, RunnableFunction,
};

#[derive(Debug)]
struct BinLength(DynBinFun);

impl RunnableFunction<u64> for BinLength {
    fn gen_value(&self, context: &mut ProgramContext) -> Result<u64, Error> {
        let bin = self.0.gen_value(context)?;
        Ok(bin.len() as u64)
    }
    fn write_value(&self, context: &mut ProgramContext, out: &mut DataGenOutput) -> Result<(), Error> {
        let value = self.gen_value(context)?;
        out.write(&value)
    }
}

fn create_bin_len(args: Arguments) -> CreateFunctionResult {
    let bin = args.required_arg("binary", 0, AnyFunction::require_bin)?;
    Ok(AnyFunction::Uint(Rc::new(BinLength(bin))))
}

pub const BIN_LENGTH: &BuiltinFunctionPrototype = &BuiltinFunctionPrototype {
    function_name: "bin_length",
    description:
        "returns the length of the given binary as a Uint. Mostly useful in mapped functions",
    arguments: &[("binary", GenType::Bin)],
    variadic: false,
    create_fn: &create_bin_len,
};

#[cfg(test)]
mod test {
    use crate::fun_test::test_program_success;

    #[test]
    fn bin_length_returns_length_of_binary() {
        let program = "bin_length(sequence([0x00, 0x01, 0x02], [0xFF, 0xFF], [0x00]))";
        let expected = "321";
        test_program_success(3, program, expected);
    }
}

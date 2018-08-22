use failure::Error;
use generator::constant::ConstantGenerator;
use generator::{DataGenRng, DynSignedIntGenerator, Generator, GeneratorArg, GeneratorType};
use interpreter::{
    ArgsBuilder, BuiltinFunctionCreator, FunctionArgs, ProgramContext,
};
use rand::prelude::Rng;
use std::fmt::{self, Display};
use writer::DataGenOutput;

const INT_FUNCTION_NAME: &'static str = "int";
const DEFAULT_MIN: i64 = ::std::i64::MIN;
const DEFAULT_MAX: i64 = ::std::i64::MAX;

pub struct SignedIntGenerator {
    min: DynSignedIntGenerator,
    max: DynSignedIntGenerator,
    value: i64,
}

impl SignedIntGenerator {
    pub fn create(min: DynSignedIntGenerator, max: DynSignedIntGenerator) -> DynSignedIntGenerator {
        Box::new(SignedIntGenerator { min, max, value: 0 })
    }
}

impl Display for SignedIntGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({}, {})", INT_FUNCTION_NAME, self.min, self.max)
    }
}

impl Generator for SignedIntGenerator {
    type Output = i64;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&i64>, Error> {
        let min = self.min.gen_value(rng)?.cloned().unwrap_or(DEFAULT_MIN);
        let max = self.max.gen_value(rng)?.cloned().unwrap_or(DEFAULT_MAX);

        if min < max {
            self.value = rng.gen_range(min, max);
        } else if min > max {
            self.value = rng.gen_range(max, min);
        } else {
            self.value = min;
        }
        Ok(Some(&self.value))
    }

    fn write_value(
        &mut self,
        rng: &mut DataGenRng,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        if let Some(val) = self.gen_value(rng)? {
            output.write_string(val).map_err(Into::into)
        } else {
            unreachable!()
        }
    }

    fn new_from_prototype(&self) -> DynSignedIntGenerator {
        let min: DynSignedIntGenerator = self.min.new_from_prototype();
        let max: DynSignedIntGenerator = self.max.new_from_prototype();
        Box::new(SignedIntGenerator { min, max, value: 0 })
    }
}

pub fn signed_int_fun2_creator() -> BuiltinFunctionCreator {
    let args = ArgsBuilder::new()
        .arg("min", GeneratorType::SignedInt)
        .arg("max", GeneratorType::SignedInt)
        .build();
    BuiltinFunctionCreator {
        name: INT_FUNCTION_NAME.into(),
        description: "generates a signed integer within the given range",
        args,
        create_fn: &create_sint2,
    }
}
fn create_sint2(mut args: Vec<GeneratorArg>, _: &ProgramContext) -> Result<GeneratorArg, Error> {
    let max = args.pop().unwrap().as_signed_int().unwrap();
    let min = args.pop().unwrap().as_signed_int().unwrap();
    Ok(GeneratorArg::SignedInt(SignedIntGenerator::create(
        min, max,
    )))
}

pub fn signed_int_min() -> BuiltinFunctionCreator {
    BuiltinFunctionCreator {
        name: "min_int".into(),
        description: "the minimum value of a signed 64 bit integer",
        args: FunctionArgs::empty(),
        create_fn: &create_min_sint,
    }
}
fn create_min_sint(_: Vec<GeneratorArg>, _: &ProgramContext) -> Result<GeneratorArg, Error> {
    Ok(GeneratorArg::SignedInt(ConstantGenerator::create(
        DEFAULT_MIN,
    )))
}

pub fn signed_int_max() -> BuiltinFunctionCreator {
    BuiltinFunctionCreator {
        name: "max_int".into(),
        description: "the maximum value of a signed 64 bit integer",
        args: FunctionArgs::empty(),
        create_fn: &create_max_sint,
    }
}
fn create_max_sint(_: Vec<GeneratorArg>, _: &ProgramContext) -> Result<GeneratorArg, Error> {
    Ok(GeneratorArg::SignedInt(ConstantGenerator::create(
        DEFAULT_MAX,
    )))
}

use super::constant::ConstantGenerator;
use super::{DataGenRng, DynUnsignedIntGenerator, Generator, GeneratorArg, GeneratorType};
use failure::Error;
use interpreter::{ArgsBuilder, BuiltinFunctionCreator, FunctionArgs, ProgramContext};
use rand::Rng;
use std::fmt;
use writer::DataGenOutput;

pub const DEFAULT_MAX: u64 = u64::max_value();
pub const DEFAULT_MIN: u64 = 0;

pub struct UnsignedIntGenerator {
    min: DynUnsignedIntGenerator,
    max: DynUnsignedIntGenerator,
    value: u64,
}

impl UnsignedIntGenerator {
    pub fn new(
        min: DynUnsignedIntGenerator,
        max: DynUnsignedIntGenerator,
    ) -> DynUnsignedIntGenerator {
        Box::new(UnsignedIntGenerator { min, max, value: 0 })
    }
}

impl Generator for UnsignedIntGenerator {
    type Output = u64;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&u64>, Error> {
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

    fn new_from_prototype(&self) -> Box<Generator<Output = u64>> {
        let min: DynUnsignedIntGenerator = self.min.new_from_prototype();
        let max: DynUnsignedIntGenerator = self.max.new_from_prototype();
        Box::new(UnsignedIntGenerator { min, max, value: 0 })
    }
}

impl fmt::Display for UnsignedIntGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unsignedInt({}, {})", self.min, self.max)
    }
}

pub fn max_uint_builtin() -> BuiltinFunctionCreator {
    BuiltinFunctionCreator {
        name: "max_uint".into(),
        description: "constant of 18,446,744,073,709,551,616 (2^64 - 1)",
        args: FunctionArgs::empty(),
        create_fn: &create_max_uint,
    }
}
fn create_max_uint(_: Vec<GeneratorArg>, _: &ProgramContext) -> Result<GeneratorArg, Error> {
    Ok(GeneratorArg::UnsignedInt(ConstantGenerator::create(
        DEFAULT_MAX,
    )))
}

pub fn uint2_builtin() -> BuiltinFunctionCreator {
    let args = ArgsBuilder::new()
        .arg("min", GeneratorType::UnsignedInt)
        .arg("max", GeneratorType::UnsignedInt)
        .build();
    BuiltinFunctionCreator {
        name: "uint".into(),
        description: "generates an unsigned integer between the given min and max",
        args,
        create_fn: &create_uint,
    }
}
fn create_uint(mut args: Vec<GeneratorArg>, _: &ProgramContext) -> Result<GeneratorArg, Error> {
    let max = args.pop().unwrap().as_uint().unwrap();
    let min = args.pop().unwrap().as_uint().unwrap();

    Ok(GeneratorArg::UnsignedInt(UnsignedIntGenerator::new(
        min, max,
    )))
}

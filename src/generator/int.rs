use failure::Error;
use generator::constant::ConstantGenerator;
use generator::{DataGenRng, DynSignedIntGenerator, Generator, GeneratorArg, GeneratorType};
use interpreter::{FunctionCreator, ProgramContext};
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

pub struct SignedIntFun2;
impl FunctionCreator for SignedIntFun2 {
    fn get_name(&self) -> &'static str {
        INT_FUNCTION_NAME
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[GeneratorType::SignedInt, GeneratorType::SignedInt], false)
    }

    fn get_description(&self) -> &'static str {
        "generates a signed integer within the given range"
    }

    fn create(
        &self,
        mut args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        let max = args.pop().unwrap().as_signed_int().unwrap();
        let min = args.pop().unwrap().as_signed_int().unwrap();
        Ok(GeneratorArg::SignedInt(SignedIntGenerator::create(
            min, max,
        )))
    }
}

pub struct SignedIntFun0;
impl FunctionCreator for SignedIntFun0 {
    fn get_name(&self) -> &'static str {
        INT_FUNCTION_NAME
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[], false)
    }

    fn get_description(&self) -> &'static str {
        "generates a signed 64 bit integer"
    }

    fn create(
        &self,
        _args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        Ok(GeneratorArg::SignedInt(SignedIntGenerator::create(
            ConstantGenerator::create(DEFAULT_MIN),
            ConstantGenerator::create(DEFAULT_MAX),
        )))
    }
}

pub struct SignedIntMin;
impl FunctionCreator for SignedIntMin {
    fn get_name(&self) -> &'static str {
        "min_int"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[], false)
    }

    fn get_description(&self) -> &'static str {
        "the minimum value of a signed 64 bit integer"
    }

    fn create(
        &self,
        _args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        Ok(GeneratorArg::SignedInt(ConstantGenerator::create(
            DEFAULT_MIN,
        )))
    }
}
pub struct SignedIntMax;
impl FunctionCreator for SignedIntMax {
    fn get_name(&self) -> &'static str {
        "max_int"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[], false)
    }

    fn get_description(&self) -> &'static str {
        "the maximum value of a signed 64 bit integer"
    }

    fn create(
        &self,
        _args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        Ok(GeneratorArg::SignedInt(ConstantGenerator::create(
            DEFAULT_MAX,
        )))
    }
}

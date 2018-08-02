use super::constant::ConstantGenerator;
use super::{DataGenRng, DynUnsignedIntGenerator, Generator, GeneratorArg, GeneratorType};
use failure::Error;
use interpreter::{FunctionCreator, ProgramContext};
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
    pub fn with_default() -> DynUnsignedIntGenerator {
        UnsignedIntGenerator::new(
            ConstantGenerator::create(DEFAULT_MIN),
            ConstantGenerator::create(DEFAULT_MAX),
        )
    }

    #[allow(dead_code)]
    pub fn with_min(min: DynUnsignedIntGenerator) -> DynUnsignedIntGenerator {
        UnsignedIntGenerator::new(min, ConstantGenerator::create(DEFAULT_MAX))
    }

    pub fn with_max(max: DynUnsignedIntGenerator) -> DynUnsignedIntGenerator {
        UnsignedIntGenerator::new(ConstantGenerator::create(DEFAULT_MIN), max)
    }

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

pub struct UnsignedInt0;
impl FunctionCreator for UnsignedInt0 {
    fn get_name(&self) -> &'static str {
        "uint"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[], false)
    }

    fn get_description(&self) -> &'static str {
        "generates an unsigned integer between 0 and 18,446,744,073,709,551,616 (2^64 - 1)"
    }

    fn create(
        &self,
        _args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        Ok(GeneratorArg::UnsignedInt(
            UnsignedIntGenerator::with_default(),
        ))
    }
}

pub struct UnsignedInt1;
impl FunctionCreator for UnsignedInt1 {
    fn get_name(&self) -> &'static str {
        "uint"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[GeneratorType::UnsignedInt], false)
    }

    fn get_description(&self) -> &'static str {
        "generates an unsigned integer between 0 and the given maximum"
    }

    fn create(
        &self,
        mut args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        let max = args.pop().unwrap().as_uint().unwrap();
        Ok(GeneratorArg::UnsignedInt(UnsignedIntGenerator::with_max(
            max,
        )))
    }
}

pub struct UnsignedInt2;
impl FunctionCreator for UnsignedInt2 {
    fn get_name(&self) -> &'static str {
        "uint"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (
            &[GeneratorType::UnsignedInt, GeneratorType::UnsignedInt],
            false,
        )
    }

    fn get_description(&self) -> &'static str {
        "generates an unsigned integer within the given range"
    }

    fn create(
        &self,
        mut args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        let max = args.pop().unwrap().as_uint().unwrap();
        let min = args.pop().unwrap().as_uint().unwrap();

        Ok(GeneratorArg::UnsignedInt(UnsignedIntGenerator::new(
            min, max,
        )))
    }
}

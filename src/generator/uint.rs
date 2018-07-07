use super::{DynUnsignedIntGenerator, Generator, DataGenRng};
use super::constant::ConstantGenerator;
use writer::DataGenOutput;
use rand::Rng;
use std::fmt;
use std::io;

pub const DEFAULT_MAX: u64 = u64::max_value();
pub const DEFAULT_MIN: u64 = 0;

pub struct UnsignedIntGenerator {
    min: DynUnsignedIntGenerator,
    max: DynUnsignedIntGenerator,
    value: u64,
}

impl UnsignedIntGenerator {

    pub fn with_default() -> DynUnsignedIntGenerator {
        UnsignedIntGenerator::new(ConstantGenerator::create(DEFAULT_MIN), ConstantGenerator::create(DEFAULT_MAX))
    }

    pub fn with_min(min: DynUnsignedIntGenerator) -> DynUnsignedIntGenerator {
        UnsignedIntGenerator::new(min, ConstantGenerator::create(DEFAULT_MAX))
    }

    pub fn with_max(max: DynUnsignedIntGenerator) -> DynUnsignedIntGenerator {
        UnsignedIntGenerator::new(ConstantGenerator::create(DEFAULT_MIN), max)
    }

    pub fn new(min: DynUnsignedIntGenerator, max: DynUnsignedIntGenerator) -> DynUnsignedIntGenerator {
        Box::new(UnsignedIntGenerator { min, max, value: 0 })
    }
}

impl Generator for UnsignedIntGenerator {
    type Output = u64;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Option<&u64> {
        let min = self.min.gen_value(rng).cloned().unwrap_or(DEFAULT_MIN);
        let max = self.max.gen_value(rng).cloned().unwrap_or(DEFAULT_MAX);

        if min < max {
            self.value = rng.gen_range(min, max);
        } else if min > max {
            self.value = rng.gen_range(max, min);
        } else {
            self.value = min;
        }
        Some(&self.value)
    }

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> io::Result<u64> {
        if let Some(val) = self.gen_value(rng) {
            output.write_string(val)
        } else {
            unreachable!()
        }
    }
}

impl fmt::Display for UnsignedIntGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unsignedInt({}, {})", self.min, self.max)
    }
}

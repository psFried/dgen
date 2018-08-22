use super::{DataGenRng, DynStringGenerator, Generator};
use failure::Error;
use std::fmt::{self, Display};
use writer::DataGenOutput;
use ::IString;

#[derive(Clone, Debug)]
pub struct ConstantGenerator<T: Display + Clone + Send> {
    value: Option<T>,
    buffer: String,
}

impl<T: Display + Clone + Send + 'static> ConstantGenerator<T> {
    pub fn new(value: Option<T>) -> ConstantGenerator<T> {
        ConstantGenerator {
            value,
            buffer: String::with_capacity(32),
        }
    }

    pub fn create(value: T) -> Box<Generator<Output = T>> {
        Box::new(ConstantGenerator::new(Some(value)))
    }
}

impl<T: Display + Clone + Send + 'static> Generator for ConstantGenerator<T> {
    type Output = T;

    fn gen_value(&mut self, _rng: &mut DataGenRng) -> Result<Option<&T>, Error> {
        Ok(self.value.as_ref())
    }

    fn write_value(
        &mut self,
        rng: &mut DataGenRng,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        if let Some(val) = self.gen_value(rng)? {
            output.write_string(val).map_err(|e| e.into())
        } else {
            Ok(0)
        }
    }

    fn new_from_prototype(&self) -> Box<Generator<Output = T>> {
        Box::new(self.clone())
    }
}

impl<T: Display + Clone + Send> Display for ConstantGenerator<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref value) = self.value {
            write!(f, "const({})", value)
        } else {
            write!(f, "const(null)")
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ConstantStringGenerator(IString);

pub fn empty_string() -> DynStringGenerator {
    ConstantStringGenerator::new("")
}

impl ConstantStringGenerator {
    pub fn new<S: Into<IString>>(value: S) -> DynStringGenerator {
        Box::new(ConstantStringGenerator(value.into()))
    }
}

impl Generator for ConstantStringGenerator {
    type Output = str;

    fn gen_value(&mut self, _rng: &mut DataGenRng) -> Result<Option<&str>, Error> {
        Ok(Some(&*self.0))
    }

    fn write_value(
        &mut self,
        _rng: &mut DataGenRng,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        output.write_string(&*self.0).map_err(|e| e.into())
    }

    fn new_from_prototype(&self) -> Box<Generator<Output = str>> {
        Box::new(self.clone())
    }
}
impl Display for ConstantStringGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "const({})", self.0)
    }
}

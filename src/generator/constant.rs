use super::{DataGenRng, Generator};
use writer::DataGenOutput;
use std::fmt::{self, Display};
use std::io;

#[derive(Clone, Debug)]
pub struct ConstantGenerator<T: Display + Clone + Send> {
    value: Option<T>,
    buffer: String,
}

impl <T: Display  + Clone + Send + 'static> ConstantGenerator<T> {
    pub fn new(value: Option<T>) -> ConstantGenerator<T> {
        ConstantGenerator {
            value,
            buffer: String::with_capacity(32),
        }
    }

    pub fn create(value: T) -> Box<Generator<Output=T>> {
        Box::new(ConstantGenerator::new(Some(value)))
    }
}

impl <T: Display + Clone + Send + 'static> Generator for ConstantGenerator<T> {
    type Output = T;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Option<&T> {
        self.value.as_ref()
    }

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> io::Result<u64> {
        if let Some(val) = self.gen_value(rng) {
            output.write_string(val)
        } else {
            Ok(0)
        }
    }

    fn new_from_prototype(&self) -> Box<Generator<Output=T>> {
        Box::new(self.clone())
    }
}

impl <T: Display + Clone + Send> Display for ConstantGenerator<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref value) = self.value {
            write!(f, "const({})", value)
        } else {
            write!(f, "const(null)")
        }
    }
}




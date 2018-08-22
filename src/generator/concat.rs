use generator::{DataGenRng, DynStringGenerator, Generator, GeneratorArg, GeneratorType};
use interpreter::{ProgramContext, ArgsBuilder, BuiltinFunctionCreator};
use writer::DataGenOutput;

use failure::Error;
use std::fmt::{self, Display};

pub const CONCAT_FUNCTION_NAME: &'static str = "concat_delimited";

pub struct ConcatFormatter {
    wrapped: Vec<DynStringGenerator>,
    value_delimeter: DynStringGenerator,
    prefix: DynStringGenerator,
    suffix: DynStringGenerator,
    buffer: String,
}

impl ConcatFormatter {
    pub fn simple(wrapped: Vec<DynStringGenerator>) -> DynStringGenerator {
        use generator::constant::empty_string;
        ConcatFormatter::new(wrapped, empty_string(), empty_string(), empty_string())
    }

    pub fn new(
        wrapped: Vec<DynStringGenerator>,
        value_delimeter: DynStringGenerator,
        prefix: DynStringGenerator,
        suffix: DynStringGenerator,
    ) -> DynStringGenerator {
        Box::new(ConcatFormatter {
            wrapped,
            value_delimeter,
            prefix,
            suffix,
            buffer: String::with_capacity(16),
        })
    }
}

fn push_str(
    buffer: &mut String,
    gen: &mut DynStringGenerator,
    rng: &mut DataGenRng,
) -> Result<(), Error> {
    if let Some(val) = gen.gen_value(rng)? {
        buffer.push_str(val);
    }
    Ok(())
}

impl Generator for ConcatFormatter {
    type Output = str;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&str>, Error> {
        let ConcatFormatter {
            ref mut wrapped,
            ref mut value_delimeter,
            ref mut prefix,
            ref mut suffix,
            ref mut buffer,
        } = *self;
        buffer.clear();

        push_str(buffer, prefix, rng)?;
        for (idx, gen) in wrapped.iter_mut().enumerate() {
            if idx > 0 {
                push_str(buffer, value_delimeter, rng)?;
            }
            push_str(buffer, gen, rng)?;
        }
        push_str(buffer, suffix, rng)?;

        Ok(Some(buffer.as_str()))
    }

    fn write_value(&mut self, rng: &mut DataGenRng, out: &mut DataGenOutput) -> Result<u64, Error> {
        let mut total = 0;
        let ConcatFormatter {
            ref mut wrapped,
            ref mut value_delimeter,
            ref mut prefix,
            ref mut suffix,
            ..
        } = *self;

        total += prefix.write_value(rng, out)?;

        for (idx, gen) in wrapped.iter_mut().enumerate() {
            if idx > 0 {
                total += value_delimeter.write_value(rng, out)?;
            }
            total += gen.write_value(rng, out)?;
        }
        total += suffix.write_value(rng, out)?;

        Ok(total)
    }

    fn new_from_prototype(&self) -> DynStringGenerator {
        let wrapped = self.wrapped
            .iter()
            .map(|g| g.new_from_prototype())
            .collect();
        let value_delimeter = self.value_delimeter.new_from_prototype();
        let prefix = self.prefix.new_from_prototype();
        let suffix = self.suffix.new_from_prototype();
        let buffer = String::with_capacity(self.buffer.capacity());
        Box::new(ConcatFormatter {
            wrapped,
            buffer,
            value_delimeter,
            prefix,
            suffix,
        })
    }
}

impl Display for ConcatFormatter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", CONCAT_FUNCTION_NAME)?;
        for (idx, gen) in self.wrapped.iter().enumerate() {
            if idx > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}", gen)?;
        }
        f.write_str(")")
    }
}

pub fn concat_delimited_function_creator() -> BuiltinFunctionCreator {
    let args = ArgsBuilder::new()
                .arg("prefix", GeneratorType::String)
                .arg("delimiter", GeneratorType::String)
                .arg("suffix", GeneratorType::String)
                .arg("values", GeneratorType::String)
                .variadic();
    BuiltinFunctionCreator {
        name: "concat_delimited".into(),
        description: "Outputs the prefix, followed by the delimited list of values, followed by the suffix",
        args,
        create_fn: &create_concat_delimited
    }
}

fn create_concat_delimited(mut args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
    let prefix = args.remove(0).as_string();
    let delimiter = args.remove(0).as_string();
    let suffix = args.remove(0).as_string();
    let values = args.into_iter().map(|g| g.as_string()).collect();
    Ok(GeneratorArg::String(ConcatFormatter::new(
        values, delimiter, prefix, suffix,
    )))
}

pub fn simple_concat_function_creator() -> BuiltinFunctionCreator {
    let args = ArgsBuilder::new()
                .arg("values", GeneratorType::String)
                .variadic();
    BuiltinFunctionCreator {
        name: "concat".into(),
        description: "Concatenates the list of inputs",
        args,
        create_fn: &create_simple_concat
    }
}

fn create_simple_concat(args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
    let values = args.into_iter().map(|g| g.as_string()).collect();
    Ok(GeneratorArg::String(ConcatFormatter::simple( values)))
}

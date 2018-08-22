use generator::{
    DataGenRng, DynGenerator, DynStringGenerator, DynUnsignedIntGenerator, Generator, GeneratorArg,
    GeneratorType,
};
use interpreter::{ProgramContext, BuiltinFunctionCreator, ArgsBuilder};
use writer::DataGenOutput;

use failure::Error;
use std::fmt::{self, Display};

pub const REPEAT_FUN_NAME: &'static str = "repeat";
pub const REPEAT_DELIMITED_FUN_NAME: &'static str = "repeat_delimited";

pub struct RepeatDelimited {
    count: DynUnsignedIntGenerator,
    repeat: DynStringGenerator,
    delimiter: DynStringGenerator,
    buffer: String,
}

impl RepeatDelimited {
    pub fn new(
        count: DynUnsignedIntGenerator,
        repeat: DynStringGenerator,
        delimiter: DynStringGenerator,
    ) -> DynStringGenerator {
        Box::new(RepeatDelimited {
            count,
            repeat,
            delimiter,
            buffer: String::new(),
        })
    }
}

impl Generator for RepeatDelimited {
    type Output = str;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&str>, Error> {
        use std::fmt::Write;
        let RepeatDelimited {
            ref mut count,
            ref mut repeat,
            ref mut delimiter,
            ref mut buffer,
        } = *self;
        buffer.clear();
        let num = count.gen_value(rng)?.cloned().unwrap_or(0);
        for i in 0..num {
            if i > 0 {
                if let Some(val) = delimiter.gen_value(rng)? {
                    buffer.write_fmt(format_args!("{}", val))?;
                }
            }
            if let Some(val) = repeat.gen_value(rng)? {
                buffer.write_fmt(format_args!("{}", val))?;
            }
        }
        Ok(Some(buffer.as_str()))
    }

    fn write_value(
        &mut self,
        rng: &mut DataGenRng,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        let num = self.count.gen_value(rng)?.cloned().unwrap_or(0);
        let mut written = 0;
        for i in 0..num {
            if i > 0 {
                written += self.delimiter.write_value(rng, output)?;
            }
            written += self.repeat.write_value(rng, output)?;
        }
        Ok(written)
    }

    fn new_from_prototype(&self) -> DynStringGenerator {
        let count = self.count.new_from_prototype();
        let repeat = self.repeat.new_from_prototype();
        let delimiter = self.delimiter.new_from_prototype();
        let buffer = String::with_capacity(self.buffer.capacity());
        Box::new(RepeatDelimited {
            count,
            repeat,
            delimiter,
            buffer,
        })
    }
}

impl Display for RepeatDelimited {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}({}, {}, {})",
            REPEAT_FUN_NAME, self.count, self.repeat, self.delimiter
        )
    }
}

pub struct Repeat<T: Display + ?Sized + 'static> {
    count: DynUnsignedIntGenerator,
    repeat: DynGenerator<T>,
    buffer: String,
}

impl<T: Display + ?Sized + 'static> Repeat<T> {
    pub fn new(count: DynUnsignedIntGenerator, repeat: DynGenerator<T>) -> DynStringGenerator {
        Box::new(Repeat {
            count,
            repeat,
            buffer: String::new(),
        })
    }
}

impl<T: Display + ?Sized + 'static> Generator for Repeat<T> {
    type Output = str;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&str>, Error> {
        use std::fmt::Write;
        let Repeat {
            ref mut count,
            ref mut repeat,
            ref mut buffer,
        } = *self;
        buffer.clear();
        let num = count.gen_value(rng)?.cloned().unwrap_or(0);
        for _ in 0..num {
            if let Some(val) = repeat.gen_value(rng)? {
                buffer.write_fmt(format_args!("{}", val))?;
            }
        }
        Ok(Some(&*buffer))
    }

    fn write_value(
        &mut self,
        rng: &mut DataGenRng,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        let num = self.count.gen_value(rng)?.cloned().unwrap_or(0);
        let mut written = 0;
        for _ in 0..num {
            written += self.repeat.write_value(rng, output)?;
        }
        Ok(written)
    }

    fn new_from_prototype(&self) -> DynStringGenerator {
        let count = self.count.new_from_prototype();
        let repeat = self.repeat.new_from_prototype();
        let buffer = String::with_capacity(self.buffer.capacity());
        Box::new(Repeat {
            count,
            repeat,
            buffer,
        })
    }
}

impl<T: Display + ?Sized + 'static> Display for Repeat<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({}, {})", REPEAT_FUN_NAME, self.count, self.repeat)
    }
}

pub fn repeat_fun() -> BuiltinFunctionCreator {
    let args = ArgsBuilder::new()
        .arg("times", GeneratorType::UnsignedInt)
        .arg("gen", GeneratorType::String)
        .build();
    BuiltinFunctionCreator {
        name: REPEAT_FUN_NAME.into(),
        description: "repeats the given generator the given number of times",
        args,
        create_fn: &create_repeat
    }
}
fn create_repeat(mut args: Vec<GeneratorArg>, _: &ProgramContext) -> Result<GeneratorArg, Error> {
    let wrapped = args.pop().unwrap().as_string();
    let count = args.pop().unwrap().as_uint().unwrap();
    Ok(GeneratorArg::String(Repeat::new(count, wrapped)))
}


pub fn repeat_delimited_fun() -> BuiltinFunctionCreator {
    let args = ArgsBuilder::new()
        .arg("times", GeneratorType::UnsignedInt)
        .arg("gen", GeneratorType::String)
        .arg("delimiter", GeneratorType::String)
        .build();
    BuiltinFunctionCreator {
        name: REPEAT_DELIMITED_FUN_NAME.into(),
        description: "repeats the given generator the given number of times, with the given delimiter in between",
        args,
        create_fn: &create_repeat_delim
    }
}
fn create_repeat_delim(mut args: Vec<GeneratorArg>, _: &ProgramContext) -> Result<GeneratorArg, Error> {
    let delimiter = args.pop().unwrap().as_string();
    let wrapped = args.pop().unwrap().as_string();
    let count = args.pop().unwrap().as_uint().unwrap();
    Ok(GeneratorArg::String(RepeatDelimited::new(
        count, wrapped, delimiter,
    )))
}

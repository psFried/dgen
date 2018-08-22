use failure::Error;
use generator::{
    DataGenRng, DynCharGenerator, DynStringGenerator, DynUnsignedIntGenerator, Generator,
    GeneratorArg, GeneratorType,
};
use interpreter::{
    ArgsBuilder, BuiltinFunctionCreator, ProgramContext,
};
use std::fmt::{self, Display};
use writer::DataGenOutput;

pub struct StringGenerator {
    char_gen: Box<Generator<Output = char>>,
    length_gen: Box<Generator<Output = u64>>,
    buffer: String,
}

impl StringGenerator {

    pub fn new(
        length_gen: DynUnsignedIntGenerator,
        char_gen: DynCharGenerator,
    ) -> DynStringGenerator {
        Box::new(StringGenerator {
            char_gen,
            length_gen,
            buffer: String::with_capacity(16),
        })
    }

    fn fill_buffer(&mut self, rng: &mut DataGenRng) -> Result<(), Error> {
        self.buffer.clear();
        let remaining_chars = self.length_gen.gen_value(rng)?.cloned().unwrap_or(16);

        for _ in 0..remaining_chars {
            let next_char = self.char_gen.gen_value(rng)?.cloned().unwrap_or('x');
            self.buffer.push(next_char);
        }
        Ok(())
    }
}

impl Generator for StringGenerator {
    type Output = str;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&str>, Error> {
        self.fill_buffer(rng)?;
        Ok(Some(&self.buffer))
    }

    fn write_value(
        &mut self,
        rng: &mut DataGenRng,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        self.fill_buffer(rng)?;
        output.write_string(&self.buffer).map_err(Into::into)
    }

    fn new_from_prototype(&self) -> Box<Generator<Output = str>> {
        let char_gen = self.char_gen.new_from_prototype();
        let length_gen = self.length_gen.new_from_prototype();
        let buffer = String::with_capacity(self.buffer.capacity());
        Box::new(StringGenerator {
            char_gen,
            length_gen,
            buffer,
        })
    }
}

impl Display for StringGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "string({}, {})", self.length_gen, self.char_gen)
    }
}

pub fn string_builtin() -> BuiltinFunctionCreator {
    let args = ArgsBuilder::new()
        .arg("length", GeneratorType::UnsignedInt)
        .arg("chars", GeneratorType::Char)
        .build();
    BuiltinFunctionCreator {
        name: "string".into(),
        description: "Generates a random string using the given length and character generators",
        args,
        create_fn: &create_string,
    }
}
fn create_string(mut args: Vec<GeneratorArg>, _: &ProgramContext) -> Result<GeneratorArg, Error> {
    let charset = args.pop().unwrap().as_char().unwrap();
    let lengeh = args.pop().unwrap().as_uint().unwrap();
    Ok(GeneratorArg::String(StringGenerator::new(lengeh, charset)))
}

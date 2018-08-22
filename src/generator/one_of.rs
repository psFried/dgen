use super::{DataGenRng, Generator, GeneratorArg, GeneratorType};
use failure::Error;
use interpreter::{
    get_bottom_argument_type, ArgsBuilder, BuiltinFunctionCreator,
    ProgramContext,
};
use rand::Rng;
use std::fmt::{self, Display};
use std::marker::PhantomData;
use writer::DataGenOutput;

pub struct OneOfGenerator<T: Display + ?Sized + 'static> {
    wrapped: Vec<Box<Generator<Output = T>>>,
    _phantom_data: PhantomData<T>,
}

impl<T: Display + ?Sized + 'static> OneOfGenerator<T> {
    pub fn new(wrapped: Vec<Box<Generator<Output = T>>>) -> Box<Generator<Output = T>> {
        Box::new(OneOfGenerator {
            wrapped,
            _phantom_data: PhantomData,
        })
    }
}

impl<T: Display + ?Sized + 'static> Generator for OneOfGenerator<T> {
    type Output = T;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&T>, Error> {
        match rng.choose_mut(self.wrapped.as_mut_slice()) {
            Some(gen) => gen.gen_value(rng),
            None => Ok(None),
        }
    }

    fn write_value(
        &mut self,
        rng: &mut DataGenRng,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        let gen = rng.choose_mut(self.wrapped.as_mut_slice());
        // gen will be None only if `wrapped` is an empty vec
        match gen {
            Some(g) => g.write_value(rng, output),
            None => Ok(0),
        }
    }

    fn new_from_prototype(&self) -> Box<Generator<Output = T>> {
        let wrapped = self
            .wrapped
            .iter()
            .map(|g| g.new_from_prototype())
            .collect::<Vec<Box<Generator<Output = T>>>>();
        Box::new(OneOfGenerator {
            wrapped,
            _phantom_data: PhantomData,
        })
    }
}

impl<T: Display + ?Sized + 'static> Display for OneOfGenerator<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("oneOf(")?;
        for (idx, gen) in self.wrapped.iter().enumerate() {
            if idx > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}", gen)?;
        }
        f.write_str(")")
    }
}

pub fn one_of_fun() -> BuiltinFunctionCreator {
    BuiltinFunctionCreator {
        name: "one_of".into(),
        description: "randomly selects one of the given arguments using a uniform distribution",
        args: ArgsBuilder::new()
            .arg("values", GeneratorType::String)
            .variadic(),
        create_fn: &create_one_of,
    }
}

fn create_one_of(args: Vec<GeneratorArg>, _: &ProgramContext) -> Result<GeneratorArg, Error> {
    let target_type = get_bottom_argument_type(args.as_slice());

    match target_type {
        GeneratorType::UnsignedInt => {
            let generators = args
                .into_iter()
                .map(|a| a.as_uint().unwrap())
                .collect::<Vec<_>>();
            Ok(GeneratorArg::UnsignedInt(OneOfGenerator::new(generators)))
        }
        GeneratorType::SignedInt => {
            let gens = args
                .into_iter()
                .map(|a| a.as_signed_int().unwrap())
                .collect::<Vec<_>>();
            Ok(GeneratorArg::SignedInt(OneOfGenerator::new(gens)))
        }
        GeneratorType::Decimal => {
            let gens = args
                .into_iter()
                .map(|a| a.as_decimal().unwrap())
                .collect::<Vec<_>>();
            Ok(GeneratorArg::Decimal(OneOfGenerator::new(gens)))
        }
        GeneratorType::Boolean => {
            let gens = args
                .into_iter()
                .map(|a| a.as_bool().unwrap())
                .collect::<Vec<_>>();
            Ok(GeneratorArg::Bool(OneOfGenerator::new(gens)))
        }
        GeneratorType::String => {
            let generators = args.into_iter().map(|a| a.as_string()).collect::<Vec<_>>();
            Ok(GeneratorArg::String(OneOfGenerator::new(generators)))
        }
        GeneratorType::Char => {
            let gens = args
                .into_iter()
                .map(|a| a.as_char().unwrap())
                .collect::<Vec<_>>();
            Ok(GeneratorArg::Char(OneOfGenerator::new(gens)))
        }
    }
}

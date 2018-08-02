use generator::{DataGenRng, DynGenerator, Generator, GeneratorArg, GeneratorType};
use interpreter::{get_bottom_argument_type, FunctionCreator, ProgramContext};
use rand::Rng;
use writer::DataGenOutput;

use failure::Error;
use std::fmt::{self, Display};

pub const STABLE_SELECT_FUN_NAME: &'static str = "stable_select";

pub struct StableSelect<T: Display + ?Sized + 'static> {
    wrapped: Vec<DynGenerator<T>>,
    index: Option<usize>,
}

impl<T: Display + ?Sized + 'static> StableSelect<T> {
    pub fn create(wrapped: Vec<DynGenerator<T>>) -> DynGenerator<T> {
        Box::new(StableSelect {
            wrapped,
            index: None,
        })
    }

    fn get_gen(&mut self, rng: &mut DataGenRng) -> &mut DynGenerator<T> {
        if self.index.is_none() {
            let i = rng.gen_range(0, self.wrapped.len());
            self.index = Some(i);
        }
        let i = self.index.unwrap();
        self.wrapped.get_mut(i).unwrap()
    }
}

impl<T: Display + ?Sized + 'static> Generator for StableSelect<T> {
    type Output = T;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&T>, Error> {
        let gen = self.get_gen(rng);
        gen.gen_value(rng)
    }

    fn write_value(
        &mut self,
        rng: &mut DataGenRng,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        let gen = self.get_gen(rng);
        gen.write_value(rng, output)
    }

    fn new_from_prototype(&self) -> DynGenerator<T> {
        let new_wrapped = self.wrapped
            .iter()
            .map(|g| g.new_from_prototype())
            .collect();
        StableSelect::create(new_wrapped)
    }
}

impl<T: Display + ?Sized + 'static> Display for StableSelect<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", STABLE_SELECT_FUN_NAME)?;
        let mut first = true;
        for gen in self.wrapped.iter() {
            if !first {
                f.write_str(", ")?;
                first = false;
            }
            write!(f, "{}", gen)?;
        }
        f.write_str(")")
    }
}

pub struct StableSelectFun;
impl FunctionCreator for StableSelectFun {
    fn get_name(&self) -> &str {
        STABLE_SELECT_FUN_NAME
    }
    fn get_arg_types(&self) -> (&[GeneratorType], bool) {
        (&[GeneratorType::String], true)
    }
    fn get_description(&self) -> &str {
        "Randomly selects one of it's inputs, and then continues to select the same one forever. Note that whichever input is selected may still generate different values."
    }

    fn create(
        &self,
        args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        let target_type = get_bottom_argument_type(args.as_slice());
        match target_type {
            GeneratorType::UnsignedInt => {
                let generators = args.into_iter()
                    .map(|a| a.as_uint().unwrap())
                    .collect::<Vec<_>>();
                Ok(GeneratorArg::UnsignedInt(StableSelect::create(generators)))
            }
            GeneratorType::SignedInt => {
                let gens = args.into_iter()
                    .map(|a| a.as_signed_int().unwrap())
                    .collect::<Vec<_>>();
                Ok(GeneratorArg::SignedInt(StableSelect::create(gens)))
            }
            GeneratorType::Decimal => {
                let gens = args.into_iter()
                    .map(|a| a.as_decimal().unwrap())
                    .collect::<Vec<_>>();
                Ok(GeneratorArg::Decimal(StableSelect::create(gens)))
            }
            GeneratorType::Boolean => {
                let gens = args.into_iter()
                    .map(|a| a.as_bool().unwrap())
                    .collect::<Vec<_>>();
                Ok(GeneratorArg::Bool(StableSelect::create(gens)))
            }
            GeneratorType::String => {
                let generators = args.into_iter().map(|a| a.as_string()).collect::<Vec<_>>();
                Ok(GeneratorArg::String(StableSelect::create(generators)))
            }
            GeneratorType::Char => {
                let gens = args.into_iter()
                    .map(|a| a.as_char().unwrap())
                    .collect::<Vec<_>>();
                Ok(GeneratorArg::Char(StableSelect::create(gens)))
            }
        }
    }
}

use super::{get_bottom_argument_type, FunctionCreator};
use failure::Error;
use generator::one_of::OneOfGenerator;
use generator::{GeneratorArg, GeneratorType};
use interpreter::resolve::ProgramContext;

// TODO: Add OneOf_ functions for other primitive types
pub struct OneOfUint;
impl FunctionCreator for OneOfUint {
    fn get_name(&self) -> &'static str {
        "one_of"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[GeneratorType::UnsignedInt], true)
    }

    fn get_description(&self) -> &'static str {
        "randomly selects one of the given arguments using a uniform distribution"
    }

    fn create(
        &self,
        args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        create_one_of(args)
    }
}

pub struct OneOfString;
impl FunctionCreator for OneOfString {
    fn get_name(&self) -> &'static str {
        "one_of"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[GeneratorType::String], true)
    }

    fn get_description(&self) -> &'static str {
        "randomly selects one of the given arguments using a uniform distribution. Allows for mixed input types"
    }

    fn create(
        &self,
        args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        create_one_of(args)
    }
}

fn create_one_of(args: Vec<GeneratorArg>) -> Result<GeneratorArg, Error> {
    let target_type = get_bottom_argument_type(args.as_slice());

    match target_type {
        GeneratorType::UnsignedInt => {
            let generators = args.into_iter()
                .map(|a| a.as_uint().unwrap())
                .collect::<Vec<_>>();
            Ok(GeneratorArg::UnsignedInt(OneOfGenerator::new(generators)))
        }
        GeneratorType::SignedInt => {
            let gens = args.into_iter()
                .map(|a| a.as_signed_int().unwrap())
                .collect::<Vec<_>>();
            Ok(GeneratorArg::SignedInt(OneOfGenerator::new(gens)))
        }
        GeneratorType::Decimal => {
            let gens = args.into_iter()
                .map(|a| a.as_decimal().unwrap())
                .collect::<Vec<_>>();
            Ok(GeneratorArg::Decimal(OneOfGenerator::new(gens)))
        }
        GeneratorType::Boolean => {
            let gens = args.into_iter()
                .map(|a| a.as_bool().unwrap())
                .collect::<Vec<_>>();
            Ok(GeneratorArg::Bool(OneOfGenerator::new(gens)))
        }
        GeneratorType::String => {
            let generators = args.into_iter().map(|a| a.as_string()).collect::<Vec<_>>();
            Ok(GeneratorArg::String(OneOfGenerator::new(generators)))
        }
        GeneratorType::Char => {
            let gens = args.into_iter()
                .map(|a| a.as_char().unwrap())
                .collect::<Vec<_>>();
            Ok(GeneratorArg::Char(OneOfGenerator::new(gens)))
        }
    }
}

use interpreter::functions::{FunctionCreator, get_bottom_argument_type};
use interpreter::resolve::ProgramContext;
use generator::{GeneratorArg, GeneratorType};
use generator::stable_select::{StableSelect, STABLE_SELECT_FUN_NAME};

use failure::Error;

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

    fn create(&self, args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
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
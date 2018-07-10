use super::FunctionCreator;
use generator::{GeneratorArg, GeneratorType};
use interpreter::resolve::ProgramContext;
use generator::concat::{CONCAT_FUNCTION_NAME, ConcatFormatter};
use failure::Error;


pub struct ConcatDelimitedFun;
impl FunctionCreator for ConcatDelimitedFun {
    fn get_name(&self) -> &str {
        CONCAT_FUNCTION_NAME
    }
    fn get_arg_types(&self) -> (&[GeneratorType], bool) {
        (&[GeneratorType::String, GeneratorType::String, GeneratorType::String, GeneratorType::String], true)
    }
    fn get_description(&self) -> &str {
        "Concatenates the inputs into a single output"
    }
    fn create(&self, mut args: Vec<GeneratorArg>, ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        let prefix = args.remove(0).as_string();
        let delimiter = args.remove(0).as_string();
        let suffix = args.remove(0).as_string();
        let values = args.into_iter().map(|g| g.as_string()).collect();
        Ok(GeneratorArg::String(ConcatFormatter::new(values, delimiter, prefix, suffix)))
    }
}

pub struct SimpleConcat;
impl FunctionCreator for SimpleConcat {
    fn get_name(&self) -> &str {
        "concat"
    }
    fn get_arg_types(&self) -> (&[GeneratorType], bool) {
        (&[GeneratorType::String], true)
    }
    fn get_description(&self) -> &str {
        "Concatenates the inputs into a single output"
    }
    fn create(&self, mut args: Vec<GeneratorArg>, ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        let prefix = args.remove(0).as_string();
        let delimiter = args.remove(0).as_string();
        let suffix = args.remove(0).as_string();
        let values = args.into_iter().map(|g| g.as_string()).collect();
        Ok(GeneratorArg::String(ConcatFormatter::new(values, delimiter, prefix, suffix)))
    }
}
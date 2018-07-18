use interpreter::functions::FunctionCreator;
use interpreter::resolve::ProgramContext;
use generator::{GeneratorType, GeneratorArg};
use generator::file::{SelectFromFile, SELECT_FROM_FILE_FUN_NAME};
use failure::Error;


pub struct SelectFromFileFun;
impl FunctionCreator for SelectFromFileFun {
    fn get_name(&self) -> &str {
        SELECT_FROM_FILE_FUN_NAME
    }
    fn get_arg_types(&self) -> (&[GeneratorType], bool) {
        (&[GeneratorType::String, GeneratorType::String], false)
    }
    fn get_description(&self) -> &str {
        "Selects random regions from the given file, using the given delimiter (most commonly a newline)"
    }
    fn create(&self, mut args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        let delimiter = args.pop().unwrap().as_string();
        let path = args.pop().unwrap().as_string();
        Ok(GeneratorArg::String(SelectFromFile::new(path, delimiter)))
    } 
}
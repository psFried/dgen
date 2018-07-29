use failure::Error;
use generator::file::{SelectFromFile, SELECT_FROM_FILE_FUN_NAME};
use generator::{GeneratorArg, GeneratorType};
use interpreter::functions::FunctionCreator;
use interpreter::resolve::ProgramContext;

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
    fn create(
        &self,
        mut args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        let delimiter = args.pop().unwrap().as_string();
        let path = args.pop().unwrap().as_string();
        Ok(GeneratorArg::String(SelectFromFile::new(path, delimiter)))
    }
}

pub struct WordsFunction;
impl FunctionCreator for WordsFunction {
    fn get_name(&self) -> &str {
        "words"
    }
    fn get_arg_types(&self) -> (&[GeneratorType], bool) {
        (&[], false)
    }
    fn get_description(&self) -> &str {
        "Selects a random word from the unix words file (/usr/share/dict/words or /usr/dict/words)"
    }
    fn create(
        &self,
        _args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
        use generator::constant::ConstantStringGenerator;
        use std::path::Path;

        let words_paths = ["/usr/share/dict/words", "/usr/dict/words"];
        let path = words_paths
            .iter()
            .filter(|path| Path::new(path).is_file())
            .next()
            .map(|path| ConstantStringGenerator::new(*path))
            .ok_or_else(|| {
                format_err!(
                    "Could not find a words file in the usual places: {:?}",
                    words_paths
                )
            })?;
        let delimiter = ConstantStringGenerator::new("\n");

        Ok(GeneratorArg::String(SelectFromFile::new(path, delimiter)))
    }
}

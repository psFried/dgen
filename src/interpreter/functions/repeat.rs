use super::FunctionCreator;
use interpreter::resolve::ProgramContext;
use generator::{GeneratorType, GeneratorArg};
use generator::repeat::{Repeat, RepeatDelimited, REPEAT_FUN_NAME, REPEAT_DELIMITED_FUN_NAME};
use failure::Error;

pub struct RepeatFun;
impl FunctionCreator for RepeatFun {
    fn get_name(&self) -> &str {
        REPEAT_FUN_NAME
    }
    fn get_arg_types(&self) -> (&[GeneratorType], bool) {
        (&[GeneratorType::UnsignedInt, GeneratorType::String], false)
    }
    fn get_description(&self) -> &str {
        "repeats the given generator the given number of times"
    }
    fn create(&self, mut args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        let wrapped = args.pop().unwrap().as_string();
        let count = args.pop().unwrap().as_uint().unwrap();
        Ok(GeneratorArg::String(Repeat::new(count, wrapped)))
    }
}

pub struct RepeatDelimitedFun;
impl FunctionCreator for RepeatDelimitedFun {
    fn get_name(&self) -> &str {
        REPEAT_DELIMITED_FUN_NAME
    }
    fn get_arg_types(&self) -> (&[GeneratorType], bool) {
        (&[GeneratorType::UnsignedInt, GeneratorType::String, GeneratorType::String], false)
    }

    fn get_description(&self) -> &str {
        "repeats the given generator the given number of times"
    }

    fn create(&self, mut args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        let delimiter = args.pop().unwrap().as_string();
        let wrapped = args.pop().unwrap().as_string();
        let count = args.pop().unwrap().as_uint().unwrap();
        Ok(GeneratorArg::String(RepeatDelimited::new(count, wrapped, delimiter)))
    }
}

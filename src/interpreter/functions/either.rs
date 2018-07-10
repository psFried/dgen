use interpreter::functions::FunctionCreator;
use interpreter::resolve::ProgramContext;
use generator::{GeneratorType, GeneratorArg};
use generator::either::{Either, EITHER_FUNCTION_NAME};
use failure::Error;



pub struct EitherFun;
impl FunctionCreator for EitherFun {
    fn get_name(&self) -> &str {
        EITHER_FUNCTION_NAME
    }
    fn get_arg_types(&self) -> (&[GeneratorType], bool) {
        (&[GeneratorType::String, GeneratorType::String], false)
    }
    fn get_description(&self) -> &str {
        "Selects either the first or the second expression, with a 50% frequency"
    }
    fn create(&self, mut args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        let b = args.pop().unwrap();
        let b_type = b.get_type();

        let a = args.pop().unwrap();
        let a_type = a.get_type();
        let result = match (a_type, b_type) {
            (GeneratorType::Boolean, GeneratorType::Boolean) => {
                GeneratorArg::Bool(Either::with_even_odds(a.as_bool().unwrap(), b.as_bool().unwrap()))
            }
            (GeneratorType::Char, GeneratorType::Char) => {
                GeneratorArg::Char(Either::with_even_odds(a.as_char().unwrap(), b.as_char().unwrap()))
            }
            (GeneratorType::UnsignedInt, GeneratorType::UnsignedInt) => {
                GeneratorArg::UnsignedInt(Either::with_even_odds(a.as_uint().unwrap(), b.as_uint().unwrap()))
            }
            (GeneratorType::SignedInt, GeneratorType::SignedInt) => {
                GeneratorArg::SignedInt(Either::with_even_odds(a.as_signed_int().unwrap(), b.as_signed_int().unwrap()))
            }
            (GeneratorType::Decimal, GeneratorType::Decimal) => {
                GeneratorArg::Decimal(Either::with_even_odds(a.as_decimal().unwrap(), b.as_decimal().unwrap()))
            }
            (_, _) => {
                GeneratorArg::String(Either::with_even_odds(a.as_string(), b.as_string()))
            }
        };
        Ok(result)
    }
}

pub struct EitherFreqFun;
impl FunctionCreator for EitherFreqFun {
    fn get_name(&self) -> &str {
        EITHER_FUNCTION_NAME
    }
    fn get_arg_types(&self) -> (&[GeneratorType], bool) {
        (&[GeneratorType::Decimal, GeneratorType::String, GeneratorType::String], false)
    }
    fn get_description(&self) -> &str {
        "Just like select2(Any, Any) except that the first argument determines the frequency with which the first possiblility will be selected"
    }
    fn create(&self, mut args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        let b = args.pop().unwrap();
        let b_type = b.get_type();

        let a = args.pop().unwrap();
        let a_type = a.get_type();
        let frequency = args.pop().unwrap().as_decimal().unwrap();

        let result = match (a_type, b_type) {
            (GeneratorType::Boolean, GeneratorType::Boolean) => {
                GeneratorArg::Bool(Either::new(frequency, a.as_bool().unwrap(), b.as_bool().unwrap()))
            }
            (GeneratorType::Char, GeneratorType::Char) => {
                GeneratorArg::Char(Either::new(frequency, a.as_char().unwrap(), b.as_char().unwrap()))
            }
            (GeneratorType::UnsignedInt, GeneratorType::UnsignedInt) => {
                GeneratorArg::UnsignedInt(Either::new(frequency, a.as_uint().unwrap(), b.as_uint().unwrap()))
            }
            (GeneratorType::SignedInt, GeneratorType::SignedInt) => {
                GeneratorArg::SignedInt(Either::new(frequency, a.as_signed_int().unwrap(), b.as_signed_int().unwrap()))
            }
            (GeneratorType::Decimal, GeneratorType::Decimal) => {
                GeneratorArg::Decimal(Either::new(frequency, a.as_decimal().unwrap(), b.as_decimal().unwrap()))
            }
            (_, _) => {
                GeneratorArg::String(Either::new(frequency, a.as_string(), b.as_string()))
            }
        };

        Ok(result)
    }
}
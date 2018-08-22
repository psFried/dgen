use generator::constant::ConstantGenerator;
use generator::{
    DataGenRng, DynDecimalGenerator, DynGenerator, Generator, GeneratorArg, GeneratorType,
};
use writer::DataGenOutput;

use failure::Error;
use interpreter::functions::BuiltinFunctionCreator;
use interpreter::{ArgsBuilder, ProgramContext};
use rand::Rng;
use std::fmt::{self, Display};

pub const EITHER_FUNCTION_NAME: &'static str = "either";
pub const MAX_FREQUENCY: f64 = 1.0;
pub const MIN_FREQUENCY: f64 = 0.0;

pub struct Either<T: Display + ?Sized + 'static> {
    a: DynGenerator<T>,
    b: DynGenerator<T>,
    a_frequency: DynDecimalGenerator,
}

impl<T: Display + ?Sized + 'static> Either<T> {
    pub fn new(
        a_frequency: DynDecimalGenerator,
        a: DynGenerator<T>,
        b: DynGenerator<T>,
    ) -> DynGenerator<T> {
        Box::new(Either { a_frequency, a, b })
    }

    fn will_select_a(&mut self, rng: &mut DataGenRng) -> Result<bool, Error> {
        let a_freq = self.a_frequency.gen_value(rng)?.cloned().unwrap_or(50.0);
        let a_freq = a_freq.min(MAX_FREQUENCY).max(MIN_FREQUENCY);
        Ok(rng.gen_bool(a_freq))
    }
}

impl<T: Display + ?Sized + 'static> Generator for Either<T> {
    type Output = T;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&T>, Error> {
        if self.will_select_a(rng)? {
            self.a.gen_value(rng)
        } else {
            self.b.gen_value(rng)
        }
    }

    fn write_value(&mut self, rng: &mut DataGenRng, out: &mut DataGenOutput) -> Result<u64, Error> {
        if self.will_select_a(rng)? {
            self.a.write_value(rng, out)
        } else {
            self.b.write_value(rng, out)
        }
    }

    fn new_from_prototype(&self) -> DynGenerator<T> {
        let a_frequency = self.a_frequency.new_from_prototype();
        let a = self.a.new_from_prototype();
        let b = self.b.new_from_prototype();
        Box::new(Either { a_frequency, a, b })
    }
}

impl<T: Display + ?Sized + 'static> Display for Either<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}({}, {}, {})",
            EITHER_FUNCTION_NAME, self.a_frequency, self.a, self.b
        )
    }
}

pub fn either_fun() -> BuiltinFunctionCreator {
    let args = ArgsBuilder::new()
        .arg("a", GeneratorType::String)
        .arg("b", GeneratorType::String)
        .build();
    let desc = "Selects either the first or the second expression, with a 50% frequency";
    BuiltinFunctionCreator::new("either", desc, args, &create_either)
}

pub fn either_freq_fun() -> BuiltinFunctionCreator {
    let args = ArgsBuilder::new()
        .arg("a_frequency", GeneratorType::Decimal)
        .arg("a", GeneratorType::String)
        .arg("b", GeneratorType::String)
        .build();
    let desc = "Selects either the first or the second expression, with a_frequencey determining the frequency of a being selected";
    BuiltinFunctionCreator::new("either", desc, args, &create_either)
}

fn create_either(mut args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
    let b = args.pop().unwrap();
    let b_type = b.get_type();

    let a = args.pop().unwrap();
    let a_type = a.get_type();

    let frequency = args.pop()
        .map(|f| f.as_decimal().unwrap())
        .unwrap_or_else(|| Box::new(ConstantGenerator::new(Some(0.5))));

    let result = match (a_type, b_type) {
        (GeneratorType::Boolean, GeneratorType::Boolean) => GeneratorArg::Bool(Either::new(
            frequency,
            a.as_bool().unwrap(),
            b.as_bool().unwrap(),
        )),
        (GeneratorType::Char, GeneratorType::Char) => GeneratorArg::Char(Either::new(
            frequency,
            a.as_char().unwrap(),
            b.as_char().unwrap(),
        )),
        (GeneratorType::UnsignedInt, GeneratorType::UnsignedInt) => GeneratorArg::UnsignedInt(
            Either::new(frequency, a.as_uint().unwrap(), b.as_uint().unwrap()),
        ),
        (GeneratorType::SignedInt, GeneratorType::SignedInt) => {
            GeneratorArg::SignedInt(Either::new(
                frequency,
                a.as_signed_int().unwrap(),
                b.as_signed_int().unwrap(),
            ))
        }
        (GeneratorType::Decimal, GeneratorType::Decimal) => GeneratorArg::Decimal(Either::new(
            frequency,
            a.as_decimal().unwrap(),
            b.as_decimal().unwrap(),
        )),
        (_, _) => GeneratorArg::String(Either::new(frequency, a.as_string(), b.as_string())),
    };
    Ok(result)
}

// pub struct EitherFun;
// impl FunctionCreator for EitherFun {
//     fn get_name(&self) -> &str {
//         EITHER_FUNCTION_NAME
//     }
//     fn get_arg_types(&self) -> (&[GeneratorType], bool) {
//         (&[GeneratorType::String, GeneratorType::String], false)
//     }
//     fn get_description(&self) -> &str {
//         "Selects either the first or the second expression, with a 50% frequency"
//     }
//     fn create(
//         &self,
//         mut args: Vec<GeneratorArg>,
//         _ctx: &ProgramContext,
//     ) -> Result<GeneratorArg, Error> {
//         let b = args.pop().unwrap();
//         let b_type = b.get_type();

//         let a = args.pop().unwrap();
//         let a_type = a.get_type();
//         let result = match (a_type, b_type) {
//             (GeneratorType::Boolean, GeneratorType::Boolean) => GeneratorArg::Bool(
//                 Either::with_even_odds(a.as_bool().unwrap(), b.as_bool().unwrap()),
//             ),
//             (GeneratorType::Char, GeneratorType::Char) => GeneratorArg::Char(
//                 Either::with_even_odds(a.as_char().unwrap(), b.as_char().unwrap()),
//             ),
//             (GeneratorType::UnsignedInt, GeneratorType::UnsignedInt) => GeneratorArg::UnsignedInt(
//                 Either::with_even_odds(a.as_uint().unwrap(), b.as_uint().unwrap()),
//             ),
//             (GeneratorType::SignedInt, GeneratorType::SignedInt) => GeneratorArg::SignedInt(
//                 Either::with_even_odds(a.as_signed_int().unwrap(), b.as_signed_int().unwrap()),
//             ),
//             (GeneratorType::Decimal, GeneratorType::Decimal) => GeneratorArg::Decimal(
//                 Either::with_even_odds(a.as_decimal().unwrap(), b.as_decimal().unwrap()),
//             ),
//             (_, _) => GeneratorArg::String(Either::with_even_odds(a.as_string(), b.as_string())),
//         };
//         Ok(result)
//     }
// }

// pub struct EitherFreqFun;
// impl FunctionCreator for EitherFreqFun {
//     fn get_name(&self) -> &str {
//         EITHER_FUNCTION_NAME
//     }
//     fn get_arg_types(&self) -> (&[GeneratorType], bool) {
//         (
//             &[
//                 GeneratorType::Decimal,
//                 GeneratorType::String,
//                 GeneratorType::String,
//             ],
//             false,
//         )
//     }
//     fn get_description(&self) -> &str {
//         "Just like select2(Any, Any) except that the first argument determines the frequency with which the first possiblility will be selected"
//     }
//     fn create(
//         &self,
//         mut args: Vec<GeneratorArg>,
//         _ctx: &ProgramContext,
//     ) -> Result<GeneratorArg, Error> {
//         let b = args.pop().unwrap();
//         let b_type = b.get_type();

//         let a = args.pop().unwrap();
//         let a_type = a.get_type();
//         let frequency = args.pop().unwrap().as_decimal().unwrap();

//         let result = match (a_type, b_type) {
//             (GeneratorType::Boolean, GeneratorType::Boolean) => GeneratorArg::Bool(Either::new(
//                 frequency,
//                 a.as_bool().unwrap(),
//                 b.as_bool().unwrap(),
//             )),
//             (GeneratorType::Char, GeneratorType::Char) => GeneratorArg::Char(Either::new(
//                 frequency,
//                 a.as_char().unwrap(),
//                 b.as_char().unwrap(),
//             )),
//             (GeneratorType::UnsignedInt, GeneratorType::UnsignedInt) => GeneratorArg::UnsignedInt(
//                 Either::new(frequency, a.as_uint().unwrap(), b.as_uint().unwrap()),
//             ),
//             (GeneratorType::SignedInt, GeneratorType::SignedInt) => {
//                 GeneratorArg::SignedInt(Either::new(
//                     frequency,
//                     a.as_signed_int().unwrap(),
//                     b.as_signed_int().unwrap(),
//                 ))
//             }
//             (GeneratorType::Decimal, GeneratorType::Decimal) => GeneratorArg::Decimal(Either::new(
//                 frequency,
//                 a.as_decimal().unwrap(),
//                 b.as_decimal().unwrap(),
//             )),
//             (_, _) => GeneratorArg::String(Either::new(frequency, a.as_string(), b.as_string())),
//         };

//         Ok(result)
//     }
// }

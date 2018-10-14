use failure::Error;
use std::fmt::{self, Display};
use std::rc::Rc;
use v2::{AnyFunction, DataGenOutput, ProgramContext, RunnableFunction};
use IString;

pub trait ResultType {
    type ValueType;

    fn get(&self) -> &Self::ValueType;
    fn get_mut(&mut self) -> &mut Self::ValueType;
}

pub trait GenTypeTrait {
    type ResultType: ResultType;

    fn get_name(&self) -> &str;
    fn to_any(&self) -> GenType;
}

// macro_rules! create_type_impl {
//     ($type_struct_name:ident, $result_struct_name:ident, $name_const:ident, $type_name:ident, $value_type:ty) => {
//         #[derive(Clone, Debug, PartialEq)]
//         pub struct $result_struct_name($value_type);
//         impl ResultType for $result_struct_name {
//             type ValueType = $value_type;

//             fn get(&self) -> &Self::ValueType {
//                 &self.0
//             }

//             fn get_mut(&mut self) -> &mut Self::ValueType {
//                 &mut self.0
//             }
//         }

//         impl Deref for $result_struct_name {
//             type Target = <Self as ResultType>::ValueType;

//             fn deref(&self) -> &Self::Target {
//                 <Self as ResultType>::get(self)
//             }
//         }

//         #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
//         pub struct $type_struct_name;
//         impl GenTypeTrait for $type_struct_name {
//             type ResultType = $result_struct_name;

//             fn get_name(&self) -> &str {
//                 stringify!($type_name)
//             }
//             fn to_any(&self) -> GenType {
//                 GenType::$type_name
//             }
//         }

//         pub const $name_const: $type_struct_name = $type_struct_name;

//         pub struct $const_struct_name($result_struct_name);

//         impl RunnableFunction<$type_struct_name> for $const_struct_name {
//             fn get_type(&self) -> $type_struct_name {
//                 $name_const
//             }

//             fn gen_value(&mut self, _context: &mut ProgramContext) -> Result<T::ResultType, Error> {

//             }

//             fn write_value(&mut self, context: &mut ProgramContext, outptu: &mut DataGenOutput) -> Result<(), Error>;
//         }
//     };
// }

// create_type_impl!(GenTypeChar, CharValue, GEN_TYPE_CHAR, Char, char);
// create_type_impl!(GenTypeString, StringValue, GEN_TYPE_STRING, String, String);
// create_type_impl!(GenTypeUint, UintValue, GEN_TYPE_UINT, Uint, u64);
// create_type_impl!(GenTypeInt, IntValue, GEN_TYPE_INT, Int, i64);
// create_type_impl!(GenTypeDecimal, DecimalValue, GEN_TYPE_DECIMAL, Decimal, f64);
// create_type_impl!(
//     GenTypeBoolean,
//     BooleanValue,
//     GEN_TYPE_BOOLEAN,
//     Boolean,
//     bool
// );

macro_rules! create_const_type {
    ($const_struct_name:ident, $output_type:ty, $any_type:path) => {
        #[derive(Debug, PartialEq, Clone)]
        pub struct $const_struct_name($output_type);

        impl RunnableFunction<$output_type> for $const_struct_name {
            fn gen_value(&self, _context: &mut ProgramContext) -> Result<$output_type, Error> {
                Ok(self.0.clone())
            }

            fn write_value(
                &self,
                _context: &mut ProgramContext,
                output: &mut DataGenOutput,
            ) -> Result<u64, Error> {
                output.write(&self.0).map_err(Into::into)
            }
        }

        impl $const_struct_name {
            #[allow(dead_code)]
            pub fn new<T: Into<$output_type>>(val: T) -> AnyFunction {
                let fun = Rc::new($const_struct_name(val.into()));
                $any_type(fun)
            }
        }
    };
}

create_const_type!(ConstBoolean, bool, AnyFunction::Boolean);
create_const_type!(ConstChar, char, AnyFunction::Char);
create_const_type!(ConstString, IString, AnyFunction::String);
create_const_type!(ConstUint, u64, AnyFunction::Uint);
create_const_type!(ConstInt, i64, AnyFunction::Int);
create_const_type!(ConstDecimal, f64, AnyFunction::Decimal);
create_const_type!(ConstBin, Vec<u8>, AnyFunction::Bin);

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum GenType {
    Char,
    String,
    Uint,
    Int,
    Decimal,
    Boolean,
    Bin,
}

impl GenType {
    pub fn display_name(&self) -> &'static str {
        match *self {
            GenType::Char => "Char",
            GenType::String => "String",
            GenType::Uint => "Uint",
            GenType::Int => "Int",
            GenType::Decimal => "Decimal",
            GenType::Boolean => "Boolean",
            GenType::Bin => "Bin",
        }
    }
}

impl Display for GenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

impl From<::generator::GeneratorType> for GenType {
    fn from(t: ::generator::GeneratorType) -> GenType {
        use generator::GeneratorType;
        match t {
            GeneratorType::Boolean => GenType::Boolean,
            GeneratorType::Char => GenType::Char,
            GeneratorType::String => GenType::String,
            GeneratorType::Decimal => GenType::Decimal,
            GeneratorType::UnsignedInt => GenType::Uint,
            GeneratorType::SignedInt => GenType::Int,
        }
    }
}

pub trait OutputType {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<u64, Error>;
}

impl OutputType for char {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<u64, Error> {
        writer.write_string(self).map_err(Into::into)
    }
}
impl OutputType for i64 {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<u64, Error> {
        writer.write_string(self).map_err(Into::into)
    }
}
impl OutputType for u64 {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<u64, Error> {
        writer.write_string(self).map_err(Into::into)
    }
}
impl OutputType for f64 {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<u64, Error> {
        writer.write_string(self).map_err(Into::into)
    }
}
impl OutputType for bool {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<u64, Error> {
        writer.write_string(self).map_err(Into::into)
    }
}
impl OutputType for IString {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<u64, Error> {
        writer.write_string(self).map_err(Into::into)
    }
}
impl OutputType for str {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<u64, Error> {
        writer.write_string(self).map_err(Into::into)
    }
}
impl OutputType for Vec<u8> {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<u64, Error> {
        writer.write_bytes(self.as_slice()).map_err(Into::into)
    }
}
impl<'a> OutputType for &'a [u8] {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<u64, Error> {
        writer.write_bytes(self).map_err(Into::into)
    }
}

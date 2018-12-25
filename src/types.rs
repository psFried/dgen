use failure::Error;
use std::rc::Rc;
use crate::IString;
use crate::{AnyFunction, DataGenOutput, ProgramContext, RunnableFunction};

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
            ) -> Result<(), Error> {
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
create_const_type!(ConstString, IString, AnyFunction::String);
create_const_type!(ConstUint, u64, AnyFunction::Uint);
create_const_type!(ConstInt, i64, AnyFunction::Int);
create_const_type!(ConstDecimal, f64, AnyFunction::Decimal);
create_const_type!(ConstBin, Vec<u8>, AnyFunction::Bin);

pub trait OutputType {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<(), Error>;
}

impl OutputType for i64 {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<(), Error> {
        writer.write_string(self)
    }
}
impl OutputType for u64 {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<(), Error> {
        writer.write_string(self)
    }
}
impl OutputType for f64 {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<(), Error> {
        writer.write_string(self)
    }
}
impl OutputType for bool {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<(), Error> {
        writer.write_string(self)
    }
}
impl OutputType for IString {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<(), Error> {
        writer.write_str(&*self)
    }
}
impl OutputType for str {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<(), Error> {
        writer.write_str(self)
    }
}
impl OutputType for Vec<u8> {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<(), Error> {
        writer.write_bytes(self.as_slice())
    }
}
impl<'a> OutputType for &'a [u8] {
    fn write_output(&self, writer: &mut DataGenOutput) -> Result<(), Error> {
        writer.write_bytes(self)
    }
}

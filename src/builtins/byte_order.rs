use byteorder::{BigEndian, ByteOrder, LittleEndian};
use failure::Error;
use std::marker::PhantomData;
use std::rc::Rc;
use {
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput, DynFun,
    FunctionPrototype, GenType, ProgramContext, RunnableFunction,
};

#[derive(Debug)]
struct NumberToBinary<T, B: ByteOrder> {
    number: DynFun<T>,
    _phantom: PhantomData<B>,
}

impl<T, B: ByteOrder> NumberToBinary<T, B> {
    fn new(number: DynFun<T>) -> NumberToBinary<T, B> {
        NumberToBinary {
            number,
            _phantom: PhantomData,
        }
    }
}

fn new_little_endian<T>(number: DynFun<T>) -> NumberToBinary<T, LittleEndian> {
    NumberToBinary::new(number)
}
fn new_big_endian<T>(number: DynFun<T>) -> NumberToBinary<T, BigEndian> {
    NumberToBinary::new(number)
}

const LITTLE_ENDIAN_FUNCTION_NAME: &str = "little_endian";
const BIG_ENDIAN_FUNCTION_NAME: &str = "big_endian";
const ARG_NAME: &str = "number";

macro_rules! impl_runnable_function {
    ($num_type:ty, $endian_type:ty, $write_bytes:path) => {
        impl RunnableFunction<Vec<u8>> for NumberToBinary<$num_type, $endian_type> {
            fn gen_value(&self, context: &mut ProgramContext) -> Result<Vec<u8>, Error> {
                let num = self.number.gen_value(context)?;
                let mut buffer = vec![0; 8];
                $write_bytes(buffer.as_mut_slice(), num);
                Ok(buffer)
            }
            fn write_value(
                &self,
                context: &mut ProgramContext,
                out: &mut DataGenOutput,
            ) -> Result<u64, Error> {
                let num = self.number.gen_value(context)?;
                let mut buffer = [0; 8];
                $write_bytes(&mut buffer[..], num);
                out.write_bytes(&buffer).map_err(Into::into)
            }
        }
    };
}

macro_rules! make_num_to_binary {
    ($le_builtin_name:ident, $be_builtin_name:ident, $gen_type:expr, $convert_input:path, $num_type:ty, $write_le_path:path, $write_be_path:path) => {
        impl_runnable_function!($num_type, LittleEndian, $write_le_path);
        impl_runnable_function!($num_type, BigEndian, $write_be_path);

        pub const $le_builtin_name: &FunctionPrototype =  {
            fn create_le(args: Arguments) -> CreateFunctionResult {
                let num = args.required_arg(ARG_NAME, 0, $convert_input)?;
                Ok(AnyFunction::Bin(Rc::new(new_little_endian(num))))
            }

            &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
                        function_name: LITTLE_ENDIAN_FUNCTION_NAME,
                        description: "converts the input number into binary with little endian byte order. The binary returned will always be exactly 8 bytes long",
                        arguments: &[
                            (ARG_NAME, $gen_type)
                        ],
                        variadic: false,
                        create_fn: &create_le,
            })
        };

        pub const $be_builtin_name: &FunctionPrototype =  {
            fn create_be(args: Arguments) -> CreateFunctionResult {
                let num = args.required_arg(ARG_NAME, 0, $convert_input)?;
                Ok(AnyFunction::Bin(Rc::new(new_big_endian(num))))
            }

            &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
                        function_name: BIG_ENDIAN_FUNCTION_NAME,
                        description: "converts the input number into binary with big endian byte order. The binary returned will always be exactly 8 bytes long",
                        arguments: &[
                            (ARG_NAME, $gen_type)
                        ],
                        variadic: false,
                        create_fn: &create_be,
            })
        };
    };
}
// ($le_builtin_name:ident, $be_builtin_name:ident, $gen_type:expr, $convert_input:path, $num_type:ty, $write_le_path:path, $write_be:path) => {

make_num_to_binary!(
    UINT_LITTLE_ENDIAN,
    UINT_BIG_ENDIAN,
    GenType::Uint,
    AnyFunction::require_uint,
    u64,
    LittleEndian::write_u64,
    BigEndian::write_u64
);
make_num_to_binary!(
    INT_LITTLE_ENDIAN,
    INT_BIG_ENDIAN,
    GenType::Int,
    AnyFunction::require_int,
    i64,
    LittleEndian::write_i64,
    BigEndian::write_i64
);
make_num_to_binary!(
    DECIMAL_LITTLE_ENDIAN,
    DECIMAL_BIG_ENDIAN,
    GenType::Decimal,
    AnyFunction::require_decimal,
    f64,
    LittleEndian::write_f64,
    BigEndian::write_f64
);

#[cfg(test)]
mod test {
    use fun_test::assert_bin_output_is_expected;

    #[test]
    fn int_is_converted_to_big_endian_binary() {
        let program = r##"big_endian(+123)"##;
        let expected = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7B];
        assert_bin_output_is_expected(program, expected);
    }

    #[test]
    fn int_is_converted_to_little_endian_binary() {
        let program = r##"little_endian(+123)"##;
        let expected = &[0x7B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_bin_output_is_expected(program, expected);
    }

    #[test]
    fn uint_is_converted_to_big_endian_binary() {
        let program = r##"big_endian(123)"##;
        let expected = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7B];
        assert_bin_output_is_expected(program, expected);
    }

    #[test]
    fn uint_is_converted_to_little_endian_binary() {
        let program = r##"little_endian(123)"##;
        let expected = &[0x7B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_bin_output_is_expected(program, expected);
    }

    #[test]
    fn decimal_is_converted_to_big_endian_binary() {
        let program = r##"big_endian(123.5)"##;
        let expected = &[0x40, 0x5E, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_bin_output_is_expected(program, expected);
    }

    #[test]
    fn decimal_is_converted_to_little_endian_binary() {
        let program = r##"little_endian(123.5)"##;
        let expected = &[0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x5E, 0x40];
        assert_bin_output_is_expected(program, expected);
    }
}

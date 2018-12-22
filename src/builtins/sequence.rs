use failure::Error;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use {
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput, DynFun,
    GenType, ProgramContext, RunnableFunction,
};

#[derive(Debug)]
struct Sequence<T> {
    values: Vec<DynFun<T>>,
    iteration: RefCell<usize>,
    wrapping: bool,
}

impl<T> Sequence<T> {
    fn next_function(&self) -> &DynFun<T> {
        let mut iterations = self.iteration.borrow_mut();

        let current = if self.wrapping {
            *iterations % self.values.len()
        } else {
            (*iterations).min(self.values.len() - 1)
        };
        *iterations += 1;
        &self.values[current]
    }
}

impl<T: Debug> RunnableFunction<T> for Sequence<T> {
    fn gen_value(&self, context: &mut ProgramContext) -> Result<T, Error> {
        self.next_function().gen_value(context)
    }
    fn write_value(
        &self,
        context: &mut ProgramContext,
        out: &mut DataGenOutput,
    ) -> Result<(), Error> {
        self.next_function().write_value(context, out)
    }
}

const SEQ_ARG_NAME: &str = "generator";

macro_rules! make_seq_builtin {
    ($wrapping_name:ident, $non_wrapping_name:ident, $gen_type:expr, $any_fun_path:path, $convert_arg:path) => {
        pub const $non_wrapping_name: &BuiltinFunctionPrototype = {

            fn create_seq(args: Arguments) -> CreateFunctionResult {
                let values = args.get_required_varargs(SEQ_ARG_NAME, 0, $convert_arg)?;
                Ok($any_fun_path(Rc::new(Sequence {
                    values,
                    iteration: RefCell::new(0),
                    wrapping: false,
                })))
            }
            &BuiltinFunctionPrototype {
                function_name: "sequence",
                description: "Iterates over the given generators in sequence. After it reaches the end, the final generator will continue to be selected forever",
                arguments: &[
                    (SEQ_ARG_NAME, $gen_type)
                ],
                variadic: true,
                create_fn: &create_seq
            }
        };

        pub const $wrapping_name: &BuiltinFunctionPrototype = {

            fn create_wrapping_seq(args: Arguments) -> CreateFunctionResult {
                let values = args.get_required_varargs(SEQ_ARG_NAME, 0, $convert_arg)?;
                Ok($any_fun_path(Rc::new(Sequence {
                    values,
                    iteration: RefCell::new(0),
                    wrapping: true,
                })))
            }
            &BuiltinFunctionPrototype {
                function_name: "wrapping_sequence",
                description: "Iterates over the given generators in sequence, wrapping around after it reaches the end",
                arguments: &[
                    (SEQ_ARG_NAME, $gen_type)
                ],
                variadic: true,
                create_fn: &create_wrapping_seq
            }
        };
    };
}

make_seq_builtin!(
    CHAR_WRAPPING_SEQ,
    CHAR_SEQ,
    GenType::Char,
    AnyFunction::Char,
    AnyFunction::require_char
);
make_seq_builtin!(
    STRING_WRAPPING_SEQ,
    STRING_SEQ,
    GenType::String,
    AnyFunction::String,
    AnyFunction::require_string
);
make_seq_builtin!(
    BIN_WRAPPING_SEQ,
    BIN_SEQ,
    GenType::Bin,
    AnyFunction::Bin,
    AnyFunction::require_bin
);
make_seq_builtin!(
    UINT_WRAPPING_SEQ,
    UINT_SEQ,
    GenType::Uint,
    AnyFunction::Uint,
    AnyFunction::require_uint
);
make_seq_builtin!(
    INT_WRAPPING_SEQ,
    INT_SEQ,
    GenType::Int,
    AnyFunction::Int,
    AnyFunction::require_int
);
make_seq_builtin!(
    DECIMAL_WRAPPING_SEQ,
    DECIMAL_SEQ,
    GenType::Decimal,
    AnyFunction::Decimal,
    AnyFunction::require_decimal
);

#[cfg(test)]
mod test {
    use fun_test::{assert_bin_output_is_expected, test_program_success};

    #[test]
    fn char_wrapping_sequence() {
        let program = r##"
        wrapping_sequence('a', 'b', 'c')
        "##;
        let expected = "abcab";
        test_program_success(5, program, expected);
    }

    #[test]
    fn char_sequence() {
        let program = r##"
        sequence('a', 'b', 'c')
        "##;
        let expected = "abccc";
        test_program_success(5, program, expected);
    }

    #[test]
    fn bin_sequence() {
        let program = r##"
        repeat(5, sequence([0x01, 0x02], [0x03, 0x04, 0x05], [0x06]))
        "##;
        let expected = &[1, 2, 3, 4, 5, 6, 6, 6];
        assert_bin_output_is_expected(program, expected);
    }

    #[test]
    fn bin_wrapping_sequence() {
        let program = r##"
        repeat(5, wrapping_sequence([0x01, 0x02], [0x03, 0x04, 0x05], [0x06]))
        "##;
        let expected = &[1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5];
        assert_bin_output_is_expected(program, expected);
    }

}

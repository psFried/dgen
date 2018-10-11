use interpreter::ast::{Expr, MacroArgument, MacroDef};
use std::fmt::{self, Debug, Display};
use v2::interpreter::Compiler;
use v2::{AnyFunction, GenType};
use IString;

use failure::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRef {
    filename: IString,
    line: u32,
    column: u32,
}

pub type CreateFunctionResult = Result<AnyFunction, Error>;

pub trait FunProto {
    fn get_name(&self) -> &str;
    fn arg_count(&self) -> usize;
    fn get_arg(&self, index: usize) -> (&str, GenType);
    fn get_description(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterpretedFunctionPrototype {
    function_name: IString,
    arguments: Vec<MacroArgument>,
    doc_comments: String, // no point in interning these
    body: Expr,
}

impl FunProto for InterpretedFunctionPrototype {
    fn get_name(&self) -> &str {
        &*self.function_name
    }
    fn arg_count(&self) -> usize {
        self.arguments.len()
    }
    fn get_arg(&self, index: usize) -> (&str, GenType) {
        let MacroArgument {
            ref name,
            ref arg_type,
        } = self.arguments[index];
        (name, (*arg_type).into())
    }
    fn get_description(&self) -> &str {
        self.doc_comments.as_str()
    }
}

impl From<MacroDef> for InterpretedFunctionPrototype {
    fn from(macro_def: MacroDef) -> InterpretedFunctionPrototype {
        InterpretedFunctionPrototype::new(macro_def)
    }
}

impl InterpretedFunctionPrototype {
    fn new(macro_def: MacroDef) -> InterpretedFunctionPrototype {
        let MacroDef {
            doc_comments,
            name,
            args,
            body,
        } = macro_def;
        InterpretedFunctionPrototype {
            function_name: name,
            doc_comments,
            arguments: args,
            body,
        }
    }
    fn bind_arguments(&self, args: &mut Vec<AnyFunction>) -> Vec<BoundArgument> {
        args.drain(..).enumerate().map(|(i, value)| {
            // the bounds check should have been taken care previously of by the type checking
            // We'll have to get a little move fancy here if we ever allow variadic functions in the grammar, though
            let arg_name  = self.arguments[i].name.clone();
            BoundArgument { arg_name, value }
        }).collect()
    }

    fn apply(&self, args: &mut Vec<AnyFunction>, compiler: &Compiler) -> CreateFunctionResult {
        let bound_args = self.bind_arguments(args);
        compiler.eval_private(&self.body, bound_args.as_slice())
    }
}

pub type BuiltinFunctionCreator = &'static Fn(&mut Vec<AnyFunction>) -> CreateFunctionResult;

pub struct BuiltinFunctionPrototype {
    pub function_name: &'static str,
    pub description: &'static str,
    pub arguments: &'static [(&'static str, GenType)],
    pub variadic: bool,
    pub create_fn: BuiltinFunctionCreator,
}

impl FunProto for BuiltinFunctionPrototype {
    fn get_name(&self) -> &str {
        &*self.function_name
    }
    fn arg_count(&self) -> usize {
        self.arguments.len()
    }
    fn get_arg(&self, index: usize) -> (&str, GenType) {
        self.arguments[index]
    }
    fn get_description(&self) -> &str {
        self.description
    }
}

impl Debug for BuiltinFunctionPrototype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let create_fn_address = format!("{:p}", self.create_fn);
        f.debug_struct("BuiltinFunctionPrototype")
            .field("function_name", &self.function_name)
            .field("arguments", &self.arguments)
            .field("variadic", &self.variadic)
            .field("create_fn", &create_fn_address)
            .finish()
    }
}

impl BuiltinFunctionPrototype {
    fn apply(&self, args: &mut Vec<AnyFunction>) -> CreateFunctionResult {
        (self.create_fn)(args)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArgumentTypes {
    pub last_is_variadic: bool,
    pub types: Vec<GenType>,
}

#[derive(Debug)]
pub enum FunctionPrototype {
    Builtin(&'static BuiltinFunctionPrototype),
    Interpreted(InterpretedFunctionPrototype),
}

impl From<InterpretedFunctionPrototype> for FunctionPrototype {
    fn from(proto: InterpretedFunctionPrototype) -> FunctionPrototype {
        FunctionPrototype::Interpreted(proto)
    }
}

impl From<&'static BuiltinFunctionPrototype> for FunctionPrototype {
    fn from(proto: &'static BuiltinFunctionPrototype) -> FunctionPrototype {
        FunctionPrototype::Builtin(proto)
    }
}

fn do_arguments_match<A: Iterator<Item = GenType>, B: Iterator<Item = GenType>>(
    mut expected_types: A,
    mut actual_types: B,
    variadic: bool,
) -> bool {
    use itertools::{EitherOrBoth, Itertools};

    let mut actual_types = actual_types.peekable();
    let mut expected_types = expected_types.peekable();

    // if this is a 0-arg function, then our job is really easy
    if expected_types.peek().is_none() {
        return actual_types.peek().is_none();
    }
    // expected_types is now guaranteed to be non-empty
    // we'll initialize last_arg_type to the first expected type here because it is considered
    // valid to pass zero arguments to a varargs function
    let mut last_arg_type = expected_types.peek().cloned().unwrap();
    for either_or_both in expected_types.zip_longest(actual_types) {
        let arg_matches = match either_or_both {
            EitherOrBoth::Both(e, a) => {
                last_arg_type = e;
                e == a
            }
            EitherOrBoth::Left(_) => false,
            EitherOrBoth::Right(arg_type) => variadic && arg_type == last_arg_type,
        };
        if !arg_matches {
            return false;
        }
    }
    true
}

impl FunctionPrototype {
    pub fn new<T: Into<InterpretedFunctionPrototype>>(t: T) -> FunctionPrototype {
        FunctionPrototype::Interpreted(t.into())
    }

    pub fn name(&self) -> &str {
        match *self {
            FunctionPrototype::Interpreted(ref int) => &*int.function_name,
            FunctionPrototype::Builtin(ref bi) => &bi.function_name,
        }
    }

    pub fn is_variadic(&self) -> bool {
        match *self {
            FunctionPrototype::Builtin(ref builtin) => builtin.variadic,
            FunctionPrototype::Interpreted(_) => false, // variadic functions are not yet supported in the grammar
        }
    }

    pub fn do_arguments_match(&self, actual_args: &[AnyFunction]) -> bool {
        let variadic = self.is_variadic();
        let actual_arg_types = actual_args.iter().map(AnyFunction::get_type);
        match *self {
            FunctionPrototype::Builtin(ref builtin) => {
                let iter = builtin.arguments.iter().map(|arg| arg.1);
                do_arguments_match(iter, actual_arg_types, variadic)
            }
            FunctionPrototype::Interpreted(ref int) => {
                let iter = int.arguments.iter().map(|arg| arg.arg_type.into());
                do_arguments_match(iter, actual_arg_types, variadic)
            }
        }
    }

    pub fn apply(
        &self,
        arguments: &mut Vec<AnyFunction>,
        compiler: &Compiler,
    ) -> CreateFunctionResult {
        match *self {
            FunctionPrototype::Builtin(ref builtin) => builtin.apply(arguments),
            FunctionPrototype::Interpreted(ref int) => int.apply(arguments, compiler),
        }
    }

    fn get_arg_count(&self) -> usize {
        match &self {
            FunctionPrototype::Builtin(ref bi) => bi.arg_count(),
            FunctionPrototype::Interpreted(ref int) => int.arg_count(),
        }
    }

    fn get_arg(&self, i: usize) -> (&str, GenType) {
        match &self {
            FunctionPrototype::Builtin(ref bi) => bi.get_arg(i),
            FunctionPrototype::Interpreted(ref int) => int.get_arg(i),
        }
    }

    fn get_description(&self) -> &str {
        match &self {
            FunctionPrototype::Builtin(ref bi) => bi.get_description(),
            FunctionPrototype::Interpreted(ref int) => int.get_description(),
        }
    }
}

impl Display for FunctionPrototype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // function name and argument list
        f.write_str(self.name())?;
        f.write_str("(")?;
        let variadic = self.is_variadic();
        let mut first = true;
        let arg_count = self.get_arg_count();
        for i in 0..arg_count {
            if !first {
                f.write_str(", ")?;
            }
            let (arg_name, arg_type) = self.get_arg(i);
            write!(f, "{}: {}", arg_name, arg_type)?;
            if variadic && i == arg_count.saturating_sub(1) {
                f.write_str(", ...")?;
            }
        }
        f.write_str(") - ")?;

        f.write_str(self.get_description())
    }
}

pub struct BoundArgument {
    pub arg_name: IString,
    pub value: AnyFunction,
}

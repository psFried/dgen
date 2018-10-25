use interpreter::ast::{WithSpan, Expr, MacroArgument, MacroDef};
use interpreter::{Source, SourceRef, Compiler, CompileResult, CompileError};
use ::{AnyFunction, Arguments, GenType};
use std::fmt::{self, Debug, Display};
use IString;
use std::sync::Arc;

use failure::Error;


pub type CreateFunctionResult = Result<AnyFunction, Error>;

pub trait FunProto {
    fn get_name(&self) -> &str;
    fn arg_count(&self) -> usize;
    fn get_arg(&self, index: usize) -> (&str, GenType);
    fn get_description(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterpretedFunctionPrototype {
    source_ref: SourceRef,
    function_name: IString,
    arguments: Vec<MacroArgument>,
    doc_comments: String, // no point in interning these
    body: WithSpan<Expr>,
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


impl InterpretedFunctionPrototype {
    pub fn new(source: Arc<Source>, macro_def: WithSpan<MacroDef>) -> InterpretedFunctionPrototype {
        let WithSpan {
            span,
            value,
        } = macro_def;

        let MacroDef {
            doc_comments,
            name,
            args,
            body,
        } = value;

        InterpretedFunctionPrototype {
            source_ref: SourceRef::new(source, span),
            function_name: name,
            doc_comments,
            arguments: args,
            body,
        }
    }

    fn bind_arguments(&self, mut args: Vec<AnyFunction>) -> Vec<BoundArgument> {
        args.drain(..).enumerate().map(|(i, value)| {
            // the bounds check should have been taken care previously of by the type checking
            // We'll have to get a little move fancy here if we ever allow variadic functions in the grammar, though
            let arg_name  = self.arguments[i].name.clone();
            BoundArgument { arg_name, value }
        }).collect()
    }

    fn apply(&self, args: Vec<AnyFunction>, compiler: &Compiler) -> CompileResult {
        let bound_args = self.bind_arguments(args);
        let source = self.source_ref.source.clone();
        compiler.eval_private(source, &self.body, bound_args.as_slice()).map_err(Into::into)
    }
}

pub type BuiltinFunctionCreator = &'static Fn(Arguments) -> CreateFunctionResult;

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
    fn apply(&self, args: Vec<AnyFunction>) -> CreateFunctionResult {
        (self.create_fn)(Arguments::new(args))
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
    expected_types: A,
    actual_types: B,
    variadic: bool,
) -> bool {
    use itertools::{EitherOrBoth, Itertools};

    let mut actual_types = actual_types.peekable();
    let mut expected_types = expected_types.peekable();

    // if this is a 0-arg function, then our job is really easy
    if expected_types.peek().is_none() {
        return actual_types.peek().is_none();
    }

    /*
     * expected_types is now guaranteed to be non-empty
     * we must require at least one argument for a varargs parameter. This is because we don't really
     * resolve to a "best" match. We instead assume that a function call will match at most two prototypes.
     * When a call does match two prototypes, we will select whichever one is NOT variadic, and error
     * if they are both variadic.
     */
    let mut last_arg_type = None;
    for either_or_both in expected_types.zip_longest(actual_types) {
        let arg_matches = match either_or_both {
            EitherOrBoth::Both(e, a) => {
                last_arg_type = Some(e);
                e == a
            }
            EitherOrBoth::Left(_) => false,
            EitherOrBoth::Right(arg_type) => variadic && Some(arg_type) == last_arg_type,
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

    pub fn is_same_signature(&self, other: &FunctionPrototype) -> bool {
        if self.name() != other.name() {
            return false;
        }

        let arg_count = self.get_arg_count();
        if arg_count != other.get_arg_count() {
            return false;
        }

        for i in 0..arg_count {
            // we're only checking the type of the arguments, not their names
            if self.get_arg(i).1 != other.get_arg(i).1 {
                return false;
            }
        }

        if self.is_variadic() != other.is_variadic() {
            return false;
        }
        true
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

    pub fn apply(&self, arguments: Vec<AnyFunction>, compiler: &Compiler, source_ref: &SourceRef) -> Result<AnyFunction, CompileError> {
        match *self {
            FunctionPrototype::Builtin(ref builtin) => {
                builtin.apply(arguments).map_err(|err| {
                    CompileError::internal_error(err, source_ref.clone())
                })
            }
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

    pub fn collect_argument_types(&self) -> Vec<GenType> {
        let arg_count = self.get_arg_count();
        let mut result = Vec::with_capacity(arg_count);
        for i in 0..arg_count {
            result.push(self.get_arg(i).1);
        }
        result
    }
}

impl Display for FunctionPrototype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // function name and argument list
        f.write_str(self.name())?;
        f.write_str("(")?;
        let variadic = self.is_variadic();
        let arg_count = self.get_arg_count();
        for i in 0..arg_count {
            if i > 0 {
                f.write_str(", ")?;
            }
            let (arg_name, arg_type) = self.get_arg(i);
            write!(f, "{}: {}", arg_name, arg_type)?;
            if variadic && i == arg_count.saturating_sub(1) {
                f.write_str("...")?;
            }
        }
        f.write_str(") - ")?;

        f.write_str(self.get_description())
    }
}

#[derive(Clone)]
pub struct BoundArgument {
    pub arg_name: IString,
    pub value: AnyFunction,
}

impl BoundArgument {
    pub fn new(arg_name: IString, value: AnyFunction) -> BoundArgument {
        BoundArgument { arg_name, value }
    }
}

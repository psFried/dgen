pub mod ast;
pub(crate) mod errors;
pub mod libraries;
mod map;
mod module;
pub(crate) mod parser;
mod source;
pub(crate) mod prototype;
mod runtime_wrapper;

#[cfg(test)]
mod parse_test;

#[allow(unused)]
pub(crate) mod grammar {
    include!(concat!(env!("OUT_DIR"), "/interpreter/grammar.rs"));
}

pub use self::source::{Source, UnreadSource};
pub use self::module::Module;
pub use self::errors::{CompileError, SourceRef};

pub const MODULE_SEPARATOR_CHAR: char = '.';

use self::ast::{Expr, FunctionCall, FunctionMapper, Program, WithSpan};
use self::map::{create_memoized_fun, finish_mapped};
use failure::Error;
use IString;
use {
    AnyFunction, BoundArgument, ConstBin, ConstBoolean, ConstChar, ConstDecimal, ConstInt,
    ConstString, ConstUint, CreateFunctionResult, FunctionPrototype,
};
use std::sync::Arc;

pub struct Compiler {
    modules: Vec<Module>,
}

pub type CompileResult = Result<AnyFunction, CompileError>;

impl Compiler {
    fn new() -> Compiler {
        Compiler {
            modules: Vec::new(),
        }
    }
    fn add_module(&mut self, module: Module) -> Result<(), Error> {
        let module_name = module.name.clone();
        
        if let Some(existing_module) = self.get_module_mut(&module_name) {
            return existing_module.combine(module);
        }  
        self.modules.push(module);
        Ok(())
    }

    fn remove_module(&mut self, module_name: &str) {
        let index = self.modules.iter().position(|m| &*m.name == module_name);
        if let Some(i) = index {
            self.modules.remove(i);
        }
    }

    pub fn eval(&self, source: Arc<Source>, expr: &WithSpan<Expr>) -> CompileResult {
        self.eval_private(source, expr, &[])
    }

    pub fn eval_private(&self, source: Arc<Source>, expr: &WithSpan<Expr>, bound_args: &[BoundArgument]) -> CompileResult {
        let source_ref = SourceRef {
            source,
            span: expr.span.clone()
        };
        match expr.value {
            Expr::Function(ref call) => self.eval_function_call(call, bound_args, source_ref),
            Expr::ArgumentUsage(ref name) => self.eval_arg_usage(name.clone(), bound_args, &source_ref),

            // literals are easy and can't really fail
            Expr::BooleanLiteral(ref lit) => Ok(ConstBoolean::new(*lit)),
            Expr::StringLiteral(ref lit) => Ok(ConstString::new(lit.clone())),
            Expr::IntLiteral(ref lit) => Ok(ConstUint::new(*lit)),
            Expr::SignedIntLiteral(ref lit) => Ok(ConstInt::new(*lit)),
            Expr::DecimalLiteral(ref lit) => Ok(ConstDecimal::new(*lit)),
            Expr::CharLiteral(ref lit) => Ok(ConstChar::new(*lit)),
            Expr::BinaryLiteral(ref lit) => Ok(ConstBin::new(lit.clone())),
        }
    }

    fn find_function<'a>(&'a self, name: IString) -> impl Iterator<Item = &'a FunctionPrototype> {
        self.modules.iter().filter_map(move |module| module.find_function(&name)).flat_map(|funs| funs)
    }

    fn function_iterator(&self) -> impl Iterator<Item = &FunctionPrototype> {
        self.modules
            .iter()
            .flat_map(|module| module.function_iterator())
    }

    fn get_module_mut(&mut self, name: &IString) -> Option<&mut Module> {
        self.modules.iter_mut().find(|module| name == &module.name)
    }

    fn get_module(&self, name: &IString, source_ref: &SourceRef) -> Result<&Module, CompileError> {
        self.modules.iter().find(|module| name == &module.name).ok_or_else(|| {
            CompileError::no_such_module(name.clone(), source_ref.clone())
        })
    }

    fn eval_function_call(
        &self,
        call: &FunctionCall,
        bound_args: &[BoundArgument],
        source_ref: SourceRef
    ) -> CompileResult {
        // first eval all the arguments for the function call and collect the results in an array
        // Any error here will short circuit the eval
        let mut resolved_args = Vec::with_capacity(call.args.len());
        for arg in call.args.iter() {
            // pass the source along when evaluating the arguments for the call
            let arg_result = self.eval_private(source_ref.source.clone(), arg, bound_args)?;
            resolved_args.push(arg_result);
        }

        // now that we have resolved all the arguments, look for a matching function prototype
        let name = call.function_name.clone();
        // first check the bound args for a matching function
        let mut resolved = bound_args
            .iter()
            .find(|bound| &*bound.arg_name == &*name)
            .map(|bound| bound.value.clone());

        if resolved.is_none() {
            let function = self.find_matching_function(source_ref.clone(), name.clone(), resolved_args.as_slice())?;
            let res = function.apply(resolved_args, self, &source_ref)?;
            resolved = Some(res);
        }
        let resolved = runtime_wrapper::wrap(resolved.unwrap(), name.clone(), source_ref.clone());

        if let Some(mapper) = call.mapper.as_ref() {
            self.eval_mapped_function(source_ref.source, resolved, mapper, bound_args)
        } else {
            Ok(resolved)
        }
    }

    fn eval_mapped_function(
        &self,
        source: Arc<Source>,
        resolved_outer: AnyFunction,
        mapper: &FunctionMapper,
        bound_args: &[BoundArgument],
    ) -> CompileResult {
        let (memoized, resetter) = create_memoized_fun(resolved_outer);
        let bound_arg = BoundArgument::new(mapper.arg_name.clone(), memoized);
        let mut all_bound_args = Vec::with_capacity(bound_args.len() + 1);
        all_bound_args.push(bound_arg);
        for arg in bound_args.iter() {
            all_bound_args.push(arg.clone());
        }

        let mapped = self.eval_private(source, &mapper.mapper_body, all_bound_args.as_slice())?;
        let resolved = finish_mapped(mapped, resetter);
        Ok(resolved)
    }

    fn find_matching_function<'a, 'b>(
        &'a self,
        caller_source_ref: SourceRef,
        name: IString,
        arguments: &'b [AnyFunction],
    ) -> Result<&'a FunctionPrototype, CompileError> {
        let name_clone = name.clone();

        if let Some((module_name, function_name)) = split_module_and_function(&*name) {
            let matching_module = self.get_module(&module_name, &caller_source_ref)?;
            let function_iter = matching_module.find_function(&function_name).ok_or_else(|| {
                CompileError::no_such_method(name, arguments, caller_source_ref.clone())
            })?;
            filter_matching_arguments(name_clone, arguments, function_iter, &caller_source_ref)
        } else {
            let iter = self.find_function(name);
            filter_matching_arguments(name_clone, arguments, iter, &caller_source_ref)
        }
    }

    fn eval_arg_usage(&self, name: IString, bound_args: &[BoundArgument], source_ref: &SourceRef) -> CompileResult {
        let bound_arg = bound_args
            .iter()
            .filter(|a| a.arg_name == name)
            .next()
            .ok_or_else(|| CompileError::no_such_argument(name, source_ref.clone()))?;

        Ok(bound_arg.value.clone())
    }
}

fn split_module_and_function(function_name: &str) -> Option<(IString, IString)> {
    function_name.rfind(MODULE_SEPARATOR_CHAR).map(|separator_position| {
        let (module, function_with_separator) = function_name.split_at(separator_position);
        let function = (&function_with_separator[1..]).into();
        (module.into(), function)
    })
}

fn set_first_empty<T>(value: T, opt1: &mut Option<T>, opt2: &mut Option<T>) {
    if opt1.is_none() {
        *opt1 = Some(value);
    } else if opt2.is_none() {
        *opt2 = Some(value)
    }
}

fn choose_best_match<'a>(opt1: Option<&'a FunctionPrototype>, opt2: Option<&'a FunctionPrototype>, called_name: &IString, actual_args: &[AnyFunction], caller_source_ref: &SourceRef) -> Result<&'a FunctionPrototype, CompileError> {
    match (opt1, opt2) {
        (Some(a), Some(b)) => {
            if a.is_variadic() && !b.is_variadic() {
                Ok(b)
            } else if !a.is_variadic() && b.is_variadic() {
                Ok(a)
            } else {
                Err(CompileError::ambiguous_function_call(called_name.clone(), actual_args, a, b, caller_source_ref.clone()))
            }
        }
        (Some(a), None) => Ok(a),
        (None, Some(b)) => Ok(b),
        (None, None) => {
            Err(CompileError::no_such_method(called_name.clone(), actual_args, caller_source_ref.clone()))
        }
    }
}

fn filter_matching_arguments<'a, I: Iterator<Item = &'a FunctionPrototype>>(name: IString, arguments: &[AnyFunction], iter: I, caller_source_ref: &SourceRef) -> Result<&'a FunctionPrototype, CompileError> {
    let mut first_from_same_module: Option<&'a FunctionPrototype> = None;
    let mut second_from_same_module: Option<&'a FunctionPrototype> = None;

    let mut first_from_other_module: Option<&'a FunctionPrototype> = None;
    let mut second_from_other_module: Option<&'a FunctionPrototype> = None;

    for candidate in iter {
        if candidate.do_arguments_match(arguments) {
            // whether the module of the caller is the same as the module of the candidate function
            // if the candidate has no source_ref, then it means that it's a builtin function
            let is_same_module = candidate.get_source().map(|source| source.module_name() == caller_source_ref.module_name()).unwrap_or(false);

            if is_same_module {
                set_first_empty(candidate, &mut first_from_same_module, &mut second_from_same_module);
            } else {
                set_first_empty(candidate, &mut first_from_other_module, &mut second_from_other_module);
            }
        }
    }

    if first_from_same_module.is_some() {
        choose_best_match(first_from_same_module, second_from_same_module, &name, arguments, caller_source_ref)
    } else {
        choose_best_match(first_from_other_module, second_from_other_module, &name, arguments, caller_source_ref)
    }
}

pub struct Interpreter {
    internal: Compiler,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut internal = Compiler::new();
        internal.add_module(::builtins::get_default_builtins_module()).expect("Failed to add builtins module to compiler");

        Interpreter {
            internal,
        }
    }

    pub fn add_std_lib(&mut self) {
        for lib in self::libraries::STDLIBS.iter() {
            let source = (*lib).clone();
            self.add_module(source)
                .expect("Failed to add std library. This is a bug!");
        }
    }

    pub fn add_module(&mut self, unread_source: UnreadSource) -> Result<(), Error> {
        self.eval_any(unread_source).map(|_| ())
    }

    pub fn has_module(&self, module_name: &str) -> bool {
        self.module_iterator().any(|module| &*module.name == module_name)
    }

    pub fn eval(&mut self, unread_source: UnreadSource) -> CreateFunctionResult {
        self.eval_any(unread_source).and_then(|maybe_function| {
            maybe_function.ok_or_else(|| {
                format_err!("The program does not end with an expression. An expression is required")
            })
        })
    }

    pub fn eval_any(&mut self, unread_source: UnreadSource) -> Result<Option<AnyFunction>, Error> {
        let source = Source::read(unread_source)?;
        let module_name: IString = source.module_name();
        
        let Program { assignments, expr } = {
            parser::parse_program(module_name.clone(), source.text())?
        };

        let source_ref = Arc::new(source);
        let module = Module::new(source_ref.clone(), assignments)?;

        if self.has_module(&*module_name) {
            bail!("A module with the name '{}' already exists and cannot be added twice. Consider giving the second module a different name", module_name);
        } else {
            self.internal.add_module(module)?;
        }

        if let Some(expression) = expr {
            let function = self.internal.eval(source_ref, &expression)?;
            Ok(Some(function))
        } else {
            Ok(None)
        }
    }

    pub fn remove_module(&mut self, name: &str) {
        self.internal.remove_module(name);
    }

    pub fn function_iterator(&self) -> impl Iterator<Item = &FunctionPrototype> {
        self.internal.function_iterator()
    }

    pub fn module_iterator(&self) -> impl Iterator<Item = &Module> {
        self.internal.modules.iter()
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use interpreter::errors::ErrorType;

    macro_rules! assert_matches {
        ($value:expr, $pattern:pat) => {
            match $value {
                $pattern => {}
                ref other @ _ => {
                    let expected = stringify!($pattern);
                    panic!("Expected: '{}' but was: {:?}", expected, other)
                }
            }
        };
    }
    
    #[test]
    fn ambiguous_calls_from_separate_modules() {
        let lib1 = r##"
        def foo(val: String) = val() { s ->
            concat(s, s)
        }; "##;
        let lib2 = r##"
        def foo(val: String) = concat("Two times the ", val, "!"); 
        "##;

        let mut subject = Interpreter::new();
        subject.add_module(UnreadSource::Builtin("lib1", lib1)).expect("failed to add module");
        subject.add_module(UnreadSource::Builtin("lib2", lib2)).expect("failed to add module");

        // calling just plain "foo" should fail
        let error = subject.eval(UnreadSource::Builtin("fail", r#"foo("wat?")"#)).expect_err("expected an error");
        let compile_error = error.downcast::<CompileError>().expect("expected a compile error");
        let error_type = compile_error.get_type();
        assert_matches!(*error_type, ErrorType::AmbiguousFunctionCall(_));

        // calling lib2.foo should compile 
        let function = subject.eval(UnreadSource::Builtin("pass", r#"lib2.foo("wat?")"#)).expect("expected compilation to succeed");
        let result = run_function(&function);
        assert_eq!("Two times the wat?!", result.as_str());
    }
    
    #[test]
    fn calling_function_with_same_name_from_same_file_resolves_to_function_in_same_file() {
        let lib1 = r##"
        def foo(val: String) = val() { s ->
            concat(s, s)
        }; "##;
        let lib2 = r##"
        def foo(val: String) = concat("Two times the ", val, "!"); 

        def bar(s: String) = foo(s);
        "##;

        let mut subject = Interpreter::new();
        subject.add_module(UnreadSource::Builtin("lib1", lib1)).expect("failed to add module");
        subject.add_module(UnreadSource::Builtin("lib2", lib2)).expect("failed to add module");

        let function = subject.eval(UnreadSource::Builtin("pass", r#"bar("wat?")"#)).expect("expected compilation to succeed");
        let result = run_function(&function);
        assert_eq!("Two times the wat?!", result.as_str());
    }

    #[test]
    fn adding_the_same_library_twice_returns_error() {
        let lib1 = r##"
        def foo() = "foo";
        "##;
        let lib2 = r##"
        def bar() = "bar";
        "##;

        let mut subject = Interpreter::new();
        subject.add_module(UnreadSource::Builtin("same_module", lib1)).expect("failed to add module");
        let error = subject.add_module(UnreadSource::Builtin("same_module", lib2)).expect_err("Expected adding second module to return an error");
        let error_message = format!("{}", error);
        assert!(error_message.contains("A module with the name 'same_module' already exists"), "wrong error message, actual: '{}'", error_message);
    }

    fn run_function(function: &AnyFunction) -> String {
        use ::{DataGenOutput, ProgramContext};

        let mut buffer = Vec::new();
        {
            let mut out = DataGenOutput::new(&mut buffer);
            let mut context = ProgramContext::from_random_seed(::verbosity::NORMAL);
            function.write_value(&mut context, &mut out).expect("failed to run function");
        }
        String::from_utf8(buffer).expect("Result was not valid utf8")
    }
}
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
use builtins::BUILTIN_FNS;
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
        let builtin_iter = builtin_functions(name.clone());
        let module_iter = self.modules.iter().filter_map(move |module| module.find_function(&name)).flat_map(|funs| funs);
        builtin_iter.chain(module_iter)
    }

    fn function_iterator(&self) -> impl Iterator<Item = &FunctionPrototype> {
        self.modules
            .iter()
            .flat_map(|module| module.function_iterator())
            .chain(BUILTIN_FNS.iter().map(|f| *f))
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
        source_ref: SourceRef,
        name: IString,
        arguments: &'b [AnyFunction],
    ) -> Result<&'a FunctionPrototype, CompileError> {
        let name_clone = name.clone();

        if let Some((module_name, function_name)) = split_module_and_function(&*name) {
            let matching_module = self.get_module(&module_name, &source_ref)?;
            let function_iter = matching_module.find_function(&function_name).ok_or_else(|| {
                CompileError::no_such_method(name, arguments, source_ref.clone())
            })?;
            filter_matching_arguments(name_clone, arguments, function_iter, &source_ref)
        } else {
            let iter = self.find_function(name);
            filter_matching_arguments(name_clone, arguments, iter, &source_ref)
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

fn builtin_functions<'a>(name: IString) -> impl Iterator<Item = &'a FunctionPrototype> {
    BUILTIN_FNS.iter().filter(move |fun| fun.name() == &*name).map(|f| *f)
}

fn filter_matching_arguments<'a, I: Iterator<Item = &'a FunctionPrototype>>(name: IString, arguments: &[AnyFunction], iter: I, source_ref: &SourceRef) -> Result<&'a FunctionPrototype, CompileError> {
    let mut current_best: Option<&'a FunctionPrototype> = None;

    for candidate in iter {
        if candidate.do_arguments_match(arguments) {
            if !candidate.is_variadic() {
                return Ok(candidate);
            } else if current_best.is_none() {
                current_best = Some(candidate);
            } else {
                let option1 = current_best.unwrap();

                return Err(CompileError::ambiguous_varargs_functions(
                    name, arguments, option1, candidate, source_ref.clone()
                ));
            }
        }
    }
    current_best.ok_or_else(|| {
        CompileError::no_such_method(name, arguments, source_ref.clone())
    })
}

pub struct Interpreter {
    internal: Compiler,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            internal: Compiler::new(),
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
        let source = Source::read(unread_source)?;
        let module_name: IString = source.module_name();
        let parsed = {
            parser::parse_program(module_name, source.text())?
        };
        let module = Module::new(Arc::new(source), parsed.assignments)?;
        self.internal.add_module(module)
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
        self.internal.add_module(module)?;

        if let Some(expression) = expr {
            let function = self.internal.eval(source_ref, &expression)?;
            Ok(Some(function))
        } else {
            Ok(None)
        }
    }

    pub fn function_iterator(&self) -> impl Iterator<Item = &FunctionPrototype> {
        self.internal.function_iterator()
    }
}

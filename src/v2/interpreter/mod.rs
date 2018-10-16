pub mod ast;
mod errors;
pub mod libraries;
mod map;
pub(crate) mod parser;
mod source;

#[cfg(test)]
mod parse_test;

#[allow(unused)]
pub(crate) mod grammar {
    include!(concat!(env!("OUT_DIR"), "/v2/interpreter/grammar.rs"));
}

use self::ast::{Expr, FunctionCall, FunctionMapper, MacroDef, Program};
use self::map::{create_memoized_fun, finish_mapped};
pub use self::source::Source;
use failure::Error;
use v2::builtins::BUILTIN_FNS;
use v2::{
    AnyFunction, BoundArgument, ConstBoolean, ConstChar, ConstDecimal, ConstInt, ConstString,
    ConstUint, CreateFunctionResult, FunctionPrototype,
};
use IString;

pub struct Module {
    _name: IString,
    functions: Vec<FunctionPrototype>,
}

impl Module {
    pub fn new(name: IString, function_defs: Vec<MacroDef>) -> Module {
        let functions = function_defs
            .into_iter()
            .map(FunctionPrototype::new)
            .collect();
        Module {
            _name: name,
            functions,
        }
    }

    fn function_iterator(&self) -> impl Iterator<Item = &FunctionPrototype> {
        self.functions.iter()
    }
}

pub struct Compiler {
    modules: Vec<Module>,
}

impl Compiler {
    fn new() -> Compiler {
        Compiler {
            modules: Vec::new(),
        }
    }
    fn add_module(&mut self, module: Module) {
        self.modules.push(module);
    }

    pub fn eval(&self, expr: &Expr) -> CreateFunctionResult {
        self.eval_private(expr, &[])
    }

    pub fn eval_private(&self, expr: &Expr, bound_args: &[BoundArgument]) -> CreateFunctionResult {
        match *expr {
            Expr::ArgumentUsage(ref name) => self.eval_arg_usage(name.clone(), bound_args),
            Expr::BooleanLiteral(ref lit) => Ok(ConstBoolean::new(*lit)),
            Expr::StringLiteral(ref lit) => Ok(ConstString::new(lit.clone())),
            Expr::IntLiteral(ref lit) => Ok(ConstUint::new(*lit)),
            Expr::SignedIntLiteral(ref lit) => Ok(ConstInt::new(*lit)),
            Expr::DecimalLiteral(ref lit) => Ok(ConstDecimal::new(*lit)),
            Expr::CharLiteral(ref lit) => Ok(ConstChar::new(*lit)),
            Expr::Function(ref call) => self.eval_function_call(call, bound_args),
        }
    }

    pub fn function_iterator(&self) -> impl Iterator<Item = &FunctionPrototype> {
        self.modules
            .iter()
            .flat_map(|module| module.function_iterator())
            .chain(BUILTIN_FNS.iter().map(|f| *f))
    }

    fn eval_function_call(
        &self,
        call: &FunctionCall,
        bound_args: &[BoundArgument],
    ) -> CreateFunctionResult {
        // first eval all the arguments for the function call and collect the results in an array
        // Any error here will short circuit the eval
        let mut resolved_args = Vec::with_capacity(call.args.len());
        for arg in call.args.iter() {
            let arg_result = self.eval_private(arg, bound_args)?;
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
            let function = self.find_function(name, resolved_args.as_slice())?;
            let res = function.apply(resolved_args, self)?;
            resolved = Some(res);
        }
        let resolved = resolved.unwrap();

        if let Some(mapper) = call.mapper.as_ref() {
            self.eval_mapped_function(resolved, mapper, bound_args)
        } else {
            Ok(resolved)
        }
    }

    fn eval_mapped_function(
        &self,
        resolved_outer: AnyFunction,
        mapper: &FunctionMapper,
        bound_args: &[BoundArgument],
    ) -> CreateFunctionResult {
        let (memoized, resetter) = create_memoized_fun(resolved_outer);
        let bound_arg = BoundArgument::new(mapper.arg_name.clone(), memoized);
        let mut all_bound_args = Vec::with_capacity(bound_args.len() + 1);
        all_bound_args.push(bound_arg);
        for arg in bound_args.iter() {
            all_bound_args.push(arg.clone());
        }

        let mapped = self.eval_private(&mapper.mapper_body, all_bound_args.as_slice())?;
        let resolved = finish_mapped(mapped, resetter);
        Ok(resolved)
    }

    fn find_function<'a, 'b>(
        &'a self,
        name: IString,
        arguments: &'b [AnyFunction],
    ) -> Result<&'a FunctionPrototype, Error> {
        let name_clone = name.clone();
        let best_match: Option<&'a FunctionPrototype> = {
            let iter = self
                .modules
                .iter()
                .rev()
                .flat_map(|module| module.function_iterator())
                .chain(BUILTIN_FNS.iter().map(|f| *f))
                .filter(|proto| proto.name() == &*name_clone);

            let mut current_best: Option<&'a FunctionPrototype> = None;

            for candidate in iter {
                if candidate.do_arguments_match(arguments) {
                    if !candidate.is_variadic() {
                        return Ok(candidate);
                    } else if current_best.is_none() {
                        current_best = Some(candidate);
                    } else {
                        let option1 = current_best.unwrap();

                        return Err(errors::ambiguous_varargs_functions(
                            name, arguments, option1, candidate,
                        ));
                    }
                }
            }
            current_best
        };

        best_match.ok_or_else(|| errors::no_such_method(name, arguments))
    }

    fn eval_arg_usage(&self, name: IString, bound_args: &[BoundArgument]) -> CreateFunctionResult {
        let bound_arg = bound_args
            .iter()
            .filter(|a| a.arg_name == name)
            .next()
            .ok_or_else(|| errors::no_such_argument(name))?;

        Ok(bound_arg.value.clone())
    }
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
            self.add_module(lib)
                .expect("Failed to add std library. This is a bug!");
        }
    }

    pub fn add_module(&mut self, source: &Source) -> Result<(), Error> {
        use std::borrow::Borrow;

        let module_name: IString = source.get_name().into();
        let text = source.read_to_str()?;
        let parsed = parser::parse_library(text.borrow()).map_err(|e| {
            e.context(format!(
                "Failed to parse source for module: '{}'",
                module_name.clone()
            ))
        })?;
        self.internal.add_module(Module::new(module_name, parsed));
        Ok(())
    }

    pub fn eval(&mut self, program: &str) -> CreateFunctionResult {
        let Program { assignments, expr } = parser::parse_program(program)?;
        let main_module = Module::new("main".into(), assignments);
        self.internal.add_module(main_module);
        self.internal.eval(&expr)
    }

    pub fn function_iterator(&self) -> impl Iterator<Item = &FunctionPrototype> {
        self.internal.function_iterator()
    }
}

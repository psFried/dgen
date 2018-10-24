pub mod ast;
pub(crate) mod errors;
pub mod libraries;
mod map;
mod module;
pub(crate) mod parser;
mod source;

#[cfg(test)]
mod parse_test;

#[allow(unused)]
pub(crate) mod grammar {
    include!(concat!(env!("OUT_DIR"), "/interpreter/grammar.rs"));
}

pub use self::source::Source;
pub use self::module::Module;

pub const MODULE_SEPARATOR_CHAR: char = '.';

use self::ast::{Expr, FunctionCall, FunctionMapper, Program};
use self::map::{create_memoized_fun, finish_mapped};
use builtins::BUILTIN_FNS;
use failure::Error;
use IString;
use {
    AnyFunction, BoundArgument, ConstBin, ConstBoolean, ConstChar, ConstDecimal, ConstInt,
    ConstString, ConstUint, CreateFunctionResult, FunctionPrototype,
};

pub struct Compiler {
    modules: Vec<Module>,
}

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
            Expr::BinaryLiteral(ref lit) => Ok(ConstBin::new(lit.clone())),
            Expr::Function(ref call) => self.eval_function_call(call, bound_args),
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

    fn get_module(&self, name: &IString) -> Result<&Module, Error> {
        self.modules.iter().find(|module| name == &module.name).ok_or_else(|| {
            errors::no_such_module(name.clone())
        })
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
            let function = self.find_matching_function(name, resolved_args.as_slice())?;
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

    fn find_matching_function<'a, 'b>(
        &'a self,
        name: IString,
        arguments: &'b [AnyFunction],
    ) -> Result<&'a FunctionPrototype, Error> {
        let name_clone = name.clone();

        if let Some((module_name, function_name)) = split_module_and_function(&*name) {
            let matching_module = self.get_module(&module_name)?;
            let function_iter = matching_module.find_function(&function_name).ok_or_else(|| {
                errors::no_such_method(name, arguments)
            })?;
            filter_matching_arguments(name_clone, arguments, function_iter)
        } else {
            let iter = self.find_function(name);
            filter_matching_arguments(name_clone, arguments, iter)
        }
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

fn filter_matching_arguments<'a, I: Iterator<Item = &'a FunctionPrototype>>(name: IString, arguments: &[AnyFunction], iter: I) -> Result<&'a FunctionPrototype, Error> {
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
    current_best.ok_or_else(|| {
        errors::no_such_method(name, arguments)
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
            self.add_module(lib)
                .expect("Failed to add std library. This is a bug!");
        }
    }

    pub fn add_module(&mut self, source: &Source) -> Result<(), Error> {
        use std::borrow::Borrow;

        let module_name: IString = source.get_name();
        let text = source.read_to_str()?;
        let parsed = parser::parse_program(module_name.clone(), text.borrow())?;
        let module = Module::new(module_name, parsed.assignments)?;
        self.internal.add_module(module)
    }

    pub fn eval(&mut self, program: &str) -> CreateFunctionResult {
        let module_name: IString = "main".into();
        let Program { assignments, expr } = parser::parse_program(module_name.clone(), program)?;

        if let Some(expression) = expr {
            let main_module = Module::new(module_name, assignments)?;
            self.internal.add_module(main_module)?;

            self.internal.eval(&expression)
        } else {
            bail!("The program does not end with an expression. An expression is required")
        }
    }

    pub fn eval_any(&mut self, program: &str) -> Result<Option<AnyFunction>, Error> {
        let module_name: IString = "main".into();
        let Program { assignments, expr } = parser::parse_program(module_name.clone(), program)?;
        let main_module = Module::new(module_name, assignments)?;
        self.internal.add_module(main_module)?;

        if let Some(expression) = expr {
            let function = self.internal.eval(&expression)?;
            Ok(Some(function))
        } else {
            Ok(None)
        }
    }

    pub fn function_iterator(&self) -> impl Iterator<Item = &FunctionPrototype> {
        self.internal.function_iterator()
    }
}

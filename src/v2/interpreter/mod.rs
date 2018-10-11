mod errors;

use failure::Error;
use interpreter::ast::{Expr, FunctionCall, MacroDef};
use v2::{
    AnyFunction, BoundArgument, BuiltinFunctionPrototype, ConstBoolean, ConstChar, ConstDecimal,
    ConstInt, ConstString, ConstUint, CreateFunctionResult, FunctionPrototype,
    InterpretedFunctionPrototype,
};
use IString;

pub struct Module {
    name: IString,
    functions: Vec<FunctionPrototype>,
}

impl Module {
    pub fn new(name: IString, function_defs: Vec<MacroDef>) -> Module {
        let functions = function_defs
            .into_iter()
            .map(FunctionPrototype::new)
            .collect();
        Module { name, functions }
    }

    fn function_iterator(&self, function_name: IString) -> impl Iterator<Item=&FunctionPrototype> {
        self.functions.iter().filter(move |fun| fun.name() == &*function_name)
    }
}

pub struct Compiler {
    modules: Vec<Module>,
}

impl Compiler {
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
        let function = self.find_function(name, resolved_args.as_slice())?;

        function.apply(&mut resolved_args, self)
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
                .flat_map(|module| module.function_iterator(name_clone.clone()));

            let mut current_best: Option<&'a FunctionPrototype> = None;

            for candidate in iter {
                if candidate.do_arguments_match(arguments) {
                    if !candidate.is_variadic() {
                        return Ok(candidate);
                    } else if current_best.is_none() {
                        current_best = Some(candidate);
                    } else {
                        let option1 = current_best.unwrap();

                        return Err(errors::ambiguous_varargs_functions(name, arguments, option1, candidate));
                    }
                }
            }
            current_best
        };

        best_match.ok_or_else(|| {
            errors::no_such_method(name, arguments)
        })
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

    pub fn add_module<S: Into<IString>>(&mut self, module_name: S, functions: Vec<MacroDef>) {
        self.internal
            .add_module(Module::new(module_name.into(), functions));
    }

    pub fn eval(&mut self, expr: &Expr) -> CreateFunctionResult {
        self.internal.eval(expr)
    }
}

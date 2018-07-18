use generator::{GeneratorArg, GeneratorType};
use generator::constant::ConstantGenerator;
use interpreter::ast::{Expr, FunctionCall, MacroDef, Program};
use interpreter::functions::{FunctionCreator, FunctionHelp, get_builtin_functions};
use std::fmt::{self, Display};
use failure::Error;

#[derive(Debug, Fail)]
pub struct ResolveError {
    message: &'static str,
    called_function: String,
    provided_args: Vec<GeneratorType>,
}

impl ResolveError {
    fn no_such_function_name(name: String, provided_args: Vec<GeneratorType>) -> ResolveError {
        ResolveError::new("no such function", name, provided_args)
    }

    fn mismatched_function_args(name: String, provided_args: Vec<GeneratorType>) -> ResolveError {
        ResolveError::new("invalid function arguments", name, provided_args)
    }

    fn ambiguous_function_call(name: String, provided_args: Vec<GeneratorType>) -> ResolveError {
        ResolveError::new("ambiguous call to an overloaded function", name, provided_args)
    }

    fn new(message: &'static str, called_function: String, provided_args: Vec<GeneratorType>) -> ResolveError {
        ResolveError {
            message,
            called_function,
            provided_args,
        }
    }
}

impl Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Resolve Error: {}: called '{}(", self.message, self.called_function)?;
        let mut first = true;
        for arg in self.provided_args.iter() {
            if !first {
                f.write_str(", ")?;
            } else {
                first = false;
            }
            write!(f, "{}", arg)?;
        }
        f.write_str(")'")?;
        let mut first = true;
        for matching in find_named_functions(self.called_function.as_str()) {
            if first {
                f.write_str("\nother possible functions are: \n")?;
                first = false;
            }
            write!(f, "{}\n", FunctionHelp(matching))?;
        }
        Ok(())
    }
}

fn find_named_functions<'a>(name: &'a str) -> impl Iterator<Item=&FunctionCreator> {
    get_builtin_functions().iter().filter(move |f| name == f.get_name()).map(|f| *f)
}

#[derive(Clone)]
struct FunctionMatch<'a> {
    call: &'a FunctionCreator,
    num_coerced: usize,
}


impl <'a> FunctionMatch<'a> {
    fn distance(&self) -> usize {
        let addl = if self.call.get_arg_types().1 {
            1
        } else {
            0
        };
        self.num_coerced + addl
    }

    fn create(candidate: &'a FunctionCreator, actual_args: &[GeneratorType]) -> Option<FunctionMatch<'a>> {
        let (expected_args, last_is_variadic) = candidate.get_arg_types();
        let expected_arg_count = expected_args.len();
        let actual_arg_count = actual_args.len();

        if expected_arg_count > actual_arg_count || (actual_arg_count > expected_arg_count && !last_is_variadic) {
            return None;
        }
        if expected_arg_count == 0 && actual_arg_count > 0 {
            return None;
        }
        if expected_arg_count == 0 && actual_arg_count == 0 {
            return Some(FunctionMatch {
                call: candidate,
                num_coerced: 0,
            });
        }

        // check to see if the argument types match. This is all guarded by the above checks
        let mut num_coerced = 0;
        for i in 0..expected_arg_count {
            let expected_type = expected_args[i];
            if actual_args[i] != expected_type {
                if expected_type == GeneratorType::String {
                    num_coerced += 1;
                } else {
                    // argument type does not match
                    return None;
                }
            }
        }

        // we only want to count the lifting of variadic function arguments at most once
        // that way, if there's multiple candidate functions, we don't report tons of coercions
        // Effectively, all variadic arguments count as one for the purposes of counting coercions
        let mut variadic_conversion_done = false;
        for i in expected_arg_count..actual_arg_count {
            if !last_is_variadic {
                // there's extra arguments provided, but this isn't a variadic function
                return None;
            }
            let expected_arg_type = expected_args[expected_arg_count - 1];
            if actual_args[i] != expected_arg_type {
                if expected_arg_type == GeneratorType::String {
                    if !variadic_conversion_done {
                        num_coerced += 1;
                        variadic_conversion_done = true;
                    }
                } else {
                    return None;
                }
            }
        }

        // omg finally! all the argument types check out, so the function argumennts are compatible
        Some(FunctionMatch {
            call: candidate,
            num_coerced
        })
    }
}


pub struct MacroDefFunctionCreator {
    description: String,
    macro_def: MacroDef,
    arg_types: Vec<GeneratorType>,
}

impl MacroDefFunctionCreator {
    pub fn new(mut macro_def: MacroDef) -> MacroDefFunctionCreator {
        let description = if macro_def.doc_comments.is_empty() {
            "user defined function".to_owned()
        } else {
            macro_def.doc_comments.join("\n")
        };
        macro_def.doc_comments.clear(); // just to deallocate the memory
        let arg_types = macro_def.args.iter().map(|a| a.arg_type).collect();
        MacroDefFunctionCreator {
            macro_def,
            arg_types,
            description,
        }
    }

    pub fn bind_arguments(&self, args: Vec<GeneratorArg>) -> Vec<MacroArgFunctionCreator> {
        args.into_iter().zip(self.macro_def.args.iter()).map(|(value, arg_type)| {
            MacroArgFunctionCreator::new(arg_type.name.clone(), value)
        }).collect()
    }
}

pub struct MacroArgFunctionCreator {
    name: String,
    value: GeneratorArg
}

impl MacroArgFunctionCreator {
    pub fn new(name: String, value: GeneratorArg) -> MacroArgFunctionCreator {
        MacroArgFunctionCreator {name, value}
    }
}

impl FunctionCreator for MacroArgFunctionCreator {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
    fn get_arg_types(&self) -> (&[GeneratorType], bool) {
        (&[], false)
    }
    fn get_description(&self) -> &str {
        self.name.as_str()
    }
    fn create(&self, _args: Vec<GeneratorArg>, _ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        Ok(self.value.clone())
    }
}

impl FunctionCreator for MacroDefFunctionCreator {
    fn get_name(&self) -> &str {
        self.macro_def.name.as_str()
    }

    fn get_arg_types(&self) -> (&[GeneratorType], bool) {
        (self.arg_types.as_slice(), false)
    }

    fn get_description(&self) -> &str {
        self.description.as_str()
    }

    fn create(&self, args: Vec<GeneratorArg>, ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        let bound_args = self.bind_arguments(args);
        ctx.resolve_macro_call(&self.macro_def.body, bound_args)
    }
}

pub struct ProgramContext {
    /// stack of scopes
    macros: Vec<Vec<MacroDefFunctionCreator>>,
}

impl ProgramContext {
    pub fn new() -> ProgramContext {
        ProgramContext {
            macros: Vec::new(),
        }
    }

    pub fn resolve_macro_call(&self, body: &Expr, args: Vec<MacroArgFunctionCreator>) -> Result<GeneratorArg, Error> {
        self.resolve_expr_private(body, args.as_slice())
    }

    pub fn add_lib(&mut self, lib: Vec<MacroDef>) {
        let scope = lib.into_iter().map(MacroDefFunctionCreator::new).collect();
        self.macros.push(scope);
    }

    pub fn resolve_program(&mut self, program: Program) -> Result<GeneratorArg, Error> {
        let Program {assignments, expr} = program;
        self.add_lib(assignments);
        let result = self.resolve_expr(&expr);
        self.macros.pop();
        result
    }

    pub fn resolve_expr(&self, token: &Expr) -> Result<GeneratorArg, Error> {
        self.resolve_expr_private(token, &[])
    }

    pub fn function_iter(&self) -> impl Iterator<Item = &FunctionCreator> {
        self.macros.iter().rev().flat_map(|v| v.iter())
            .map(|i| i as &FunctionCreator)
            .chain(get_builtin_functions().iter().map(|f| *f))
    }

    fn resolve_expr_private(&self, token: &Expr, bound_arguments: &[MacroArgFunctionCreator]) -> Result<GeneratorArg, Error> {
        match token {
            Expr::BooleanLiteral(val) => Ok(GeneratorArg::Bool(ConstantGenerator::create(val.clone()))),
            Expr::StringLiteral(val) => Ok(GeneratorArg::String(ConstantGenerator::create(val.clone()))),
            Expr::IntLiteral(int) => Ok(GeneratorArg::UnsignedInt(ConstantGenerator::create(int.clone()))),
            Expr::SignedIntLiteral(val) => Ok(GeneratorArg::SignedInt(ConstantGenerator::create(val.clone()))),
            Expr::DecimalLiteral(float) => Ok(GeneratorArg::Decimal(ConstantGenerator::create(float.clone()))),
            Expr::Function(call) => self.resolve_function_call(call, bound_arguments)
        }

    }

    fn get_matching_functions<'a>(&'a self, function_name: &'a str) -> impl Iterator<Item=&'a FunctionCreator> {
        let f_name = function_name;
        self.macros.iter().rev().flat_map(|v| v.iter())
            .filter(move |i| i.get_name() == function_name)
            .map(|i| i as &FunctionCreator)
            .chain(find_named_functions(f_name))
    }

    fn resolve_function_call(&self, function_call: &FunctionCall, bound_args: &[MacroArgFunctionCreator]) -> Result<GeneratorArg, Error> {
        let FunctionCall { ref function_name, ref args } = *function_call;
        let mut resolved_args = Vec::with_capacity(args.len());
        let mut resolved_argument_types: Vec<GeneratorType> = Vec::with_capacity(args.len());
        for token in args.iter() {
            let resolved_arg = self.resolve_expr_private(&token, bound_args)?;
            resolved_argument_types.push(resolved_arg.get_type());
            resolved_args.push(resolved_arg);
        }

        let matching_arg_functions = bound_args.iter()
                .filter(|a| a.get_name() == function_name.as_str())
                .map(|a| a as &FunctionCreator);

        // first find all the functions where just the name matches
        let mut matching_name_functions = matching_arg_functions.chain(self.get_matching_functions(function_name.as_str())).peekable();
        // ensure that there's at least one function somewhere with a name that matches, and return early if not
        if matching_name_functions.peek().is_none() {
            return Err(ResolveError::no_such_function_name(function_name.clone(), resolved_argument_types).into());
        }

        let matching_functions = matching_name_functions.flat_map(|fun| {
            FunctionMatch::create(fun, resolved_argument_types.as_slice())
        });

        let mut best_match = None;
        let mut current_best_distance = usize::max_value();
        // if multiple functions _could_ match, then we pick the one where the arguments match the most specifically
        for function_match in matching_functions {
            let match_distance = function_match.distance();
            if match_distance < current_best_distance {
                best_match = Some(function_match);
                current_best_distance = match_distance;
            } else if match_distance == current_best_distance {
                // 2 or more functions have the same distance, which means we have to error out
                return Err(ResolveError::ambiguous_function_call(function_name.clone(), resolved_argument_types.clone()).into());
            }
        }

        best_match.ok_or_else(|| {
            // there were no functions that matched both the name and the argument types
            ResolveError::mismatched_function_args(function_name.clone(), resolved_argument_types.clone()).into()
        }).and_then(|creator| {
            creator.call.create(resolved_args, self)
        })
    }
}
mod error;
mod interpreted_function_creator;

use failure::Error;
use generator::constant::{ConstantGenerator, ConstantStringGenerator};
use generator::{GeneratorArg, GeneratorType};
use interpreter::ast::{Expr, FunctionCall, FunctionMapper, MacroDef, Program};
use interpreter::functions::{get_builtin_functions, FunctionCreator};

use self::error::ResolveError;
use self::interpreted_function_creator::{MacroArgFunctionCreator, MacroDefFunctionCreator};

fn find_named_functions<'a>(name: &'a str) -> impl Iterator<Item = &FunctionCreator> {
    get_builtin_functions()
        .iter()
        .filter(move |f| name == f.get_name())
        .map(|f| *f)
}

#[derive(Clone)]
struct FunctionMatch<'a> {
    call: &'a FunctionCreator,
    num_coerced: usize,
}

impl<'a> FunctionMatch<'a> {
    fn distance(&self) -> usize {
        let addl = if self.call.get_arg_types().1 { 1 } else { 0 };
        self.num_coerced + addl
    }

    fn create(
        candidate: &'a FunctionCreator,
        actual_args: &[GeneratorType],
    ) -> Option<FunctionMatch<'a>> {
        let (expected_args, last_is_variadic) = candidate.get_arg_types();
        let expected_arg_count = expected_args.len();
        let actual_arg_count = actual_args.len();

        if expected_arg_count > actual_arg_count
            || (actual_arg_count > expected_arg_count && !last_is_variadic)
        {
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
            num_coerced,
        })
    }
}

pub struct ProgramContext {
    /// stack of scopes
    macros: Vec<Vec<MacroDefFunctionCreator>>,
}

impl ProgramContext {
    pub fn new() -> ProgramContext {
        ProgramContext { macros: Vec::new() }
    }

    pub fn resolve_macro_call(
        &self,
        body: &Expr,
        args: Vec<MacroArgFunctionCreator>,
    ) -> Result<GeneratorArg, Error> {
        let mut scopes = vec![args];
        self.resolve_expr_private(body, &mut scopes)
    }

    pub fn add_lib(&mut self, lib: Vec<MacroDef>) {
        let scope = lib.into_iter().map(MacroDefFunctionCreator::new).collect();
        self.macros.push(scope);
    }

    pub fn resolve_program(&mut self, program: Program) -> Result<GeneratorArg, Error> {
        let Program { assignments, expr } = program;
        self.add_lib(assignments);
        let result = self.resolve_expr(&expr);
        self.macros.pop();
        result
    }

    pub fn resolve_expr(&self, token: &Expr) -> Result<GeneratorArg, Error> {
        self.resolve_expr_private(token, &mut Vec::new())
    }

    pub fn function_iter(&self) -> impl Iterator<Item = &FunctionCreator> {
        self.macros
            .iter()
            .rev()
            .flat_map(|v| v.iter())
            .map(|i| i as &FunctionCreator)
            .chain(get_builtin_functions().iter().map(|f| *f))
    }

    fn resolve_expr_private(
        &self,
        token: &Expr,
        bound_arguments: &mut Vec<Vec<MacroArgFunctionCreator>>,
    ) -> Result<GeneratorArg, Error> {
        match token {
            Expr::CharLiteral(val) => {
                Ok(GeneratorArg::Char(ConstantGenerator::create(val.clone())))
            }
            Expr::BooleanLiteral(val) => {
                Ok(GeneratorArg::Bool(ConstantGenerator::create(val.clone())))
            }
            Expr::StringLiteral(val) => Ok(GeneratorArg::String(ConstantStringGenerator::new(
                val.clone(),
            ))),
            Expr::IntLiteral(int) => Ok(GeneratorArg::UnsignedInt(ConstantGenerator::create(
                int.clone(),
            ))),
            Expr::SignedIntLiteral(val) => Ok(GeneratorArg::SignedInt(ConstantGenerator::create(
                val.clone(),
            ))),
            Expr::DecimalLiteral(float) => Ok(GeneratorArg::Decimal(ConstantGenerator::create(
                float.clone(),
            ))),
            Expr::Function(call) => self.resolve_function_call(call, bound_arguments),
        }
    }

    fn get_matching_functions<'a>(
        &'a self,
        function_name: &'a str,
    ) -> impl Iterator<Item = &'a FunctionCreator> {
        let f_name = function_name;
        self.macros
            .iter()
            .rev()
            .flat_map(|v| v.iter())
            .filter(move |i| i.get_name() == function_name)
            .map(|i| i as &FunctionCreator)
            .chain(find_named_functions(f_name))
    }

    fn resolve_function_call(
        &self,
        function_call: &FunctionCall,
        bound_args: &mut Vec<Vec<MacroArgFunctionCreator>>,
    ) -> Result<GeneratorArg, Error> {
        let FunctionCall {
            ref function_name,
            ref args,
            ref mapper,
        } = *function_call;

        // first thing is to resolve all the arguments that were passed into the function call
        let mut resolved_args = Vec::with_capacity(args.len());
        let mut resolved_argument_types: Vec<GeneratorType> = Vec::with_capacity(args.len());
        for token in args.iter() {
            let resolved_arg = self.resolve_expr_private(&token, bound_args)?;
            resolved_argument_types.push(resolved_arg.get_type());
            resolved_args.push(resolved_arg);
        }

        let resolved_generator = {
            self.call_function_creator(
                function_name.as_str(),
                resolved_argument_types,
                resolved_args,
                bound_args,
            )?
        };

        if let Some(function_mapper) = mapper {
            self.resolve_function_mapper(resolved_generator, function_mapper, bound_args)
        } else {
            Ok(resolved_generator)
        }
    }

    fn resolve_function_mapper(
        &self,
        outer: GeneratorArg,
        mapper: &FunctionMapper,
        bound_args: &mut Vec<Vec<MacroArgFunctionCreator>>,
    ) -> Result<GeneratorArg, Error> {
        use generator::mapped::{create_arg, wrap_mapped_gen};

        let arg_name = mapper.arg_name.clone();
        let (closure_arg, resetter) = create_arg(arg_name.clone(), outer);
        let arg_creator = MacroArgFunctionCreator::new(arg_name, closure_arg);
        bound_args.push(vec![arg_creator]);

        let result = self.resolve_expr_private(&mapper.mapper_body, bound_args);
        let _ = bound_args.pop();
        let expr = result?;

        Ok(wrap_mapped_gen(expr, resetter))
    }

    // resolves the appropriate function creator and invokes it. returns an error if no function creator can be found
    // or if the function creator itself returns an error
    fn call_function_creator(
        &self,
        function_name: &str,
        resolved_argument_types: Vec<GeneratorType>,
        resolved_args: Vec<GeneratorArg>,
        bound_args: &mut Vec<Vec<MacroArgFunctionCreator>>,
    ) -> Result<GeneratorArg, Error> {
        let matching_arg_functions = bound_args
            .iter()
            .flat_map(|a| a.iter())
            .filter(|a| a.get_name() == function_name)
            .map(|a| a as &FunctionCreator);

        // first find all the functions where just the name matches
        let mut matching_name_functions = matching_arg_functions
            .chain(self.get_matching_functions(function_name))
            .peekable();
        // ensure that there's at least one function somewhere with a name that matches, and return early if not
        if matching_name_functions.peek().is_none() {
            return Err(ResolveError::no_such_function_name(
                function_name.to_owned(),
                resolved_argument_types,
            ).into());
        }

        let matching_functions = matching_name_functions
            .flat_map(|fun| FunctionMatch::create(fun, resolved_argument_types.as_slice()));

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
                return Err(ResolveError::ambiguous_function_call(
                    function_name.to_owned(),
                    resolved_argument_types.clone(),
                ).into());
            }
        }

        best_match
            .ok_or_else(|| {
                // there were no functions that matched both the name and the argument types
                ResolveError::mismatched_function_args(
                    function_name.to_owned(),
                    resolved_argument_types.clone(),
                ).into()
            })
            .and_then(|creator| creator.call.create(resolved_args, self))
    }
}

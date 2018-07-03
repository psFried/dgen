use generator::{GeneratorArg, GeneratorType};
use generator::constant::ConstantGenerator;
use ast::{Token, FunctionCall};
use functions::{FunctionCreator, ALL_FUNCTIONS};
use std::fmt::{self, Display};


// TODO: put some real info in here for christs sake
#[derive(Debug, PartialEq)]
pub struct ResolveError;

impl Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("ResolveError")
    }
}

pub fn into_generator(token: Token) -> Result<GeneratorArg, ResolveError> {
    match token {
        Token::StringLiteral(val) => Ok(GeneratorArg::String(ConstantGenerator::create(val))),
        Token::IntLiteral(int) => Ok(GeneratorArg::UnsignedInt(ConstantGenerator::create(int))),
        Token::DecimalLiteral(float) => Ok(GeneratorArg::Decimal(ConstantGenerator::create(float))),
        Token::Function(call) => resolve_function_call(call)
    }
}

fn find_named_functions(name: &str) -> impl Iterator<Item=&FunctionCreator> {
    ALL_FUNCTIONS.iter().filter(move |f| name == f.get_name()).map(|f| *f)
}


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

fn resolve_function_call(function_call: FunctionCall) -> Result<GeneratorArg, ResolveError> {
    let FunctionCall { function_name, args } = function_call;
    let mut resolved_args = Vec::with_capacity(args.len());
    let mut argument_types: Vec<GeneratorType> = Vec::with_capacity(args.len());
    for token in args {
        let resolved_arg = into_generator(token)?;
        argument_types.push(resolved_arg.get_type());
        resolved_args.push(resolved_arg);
    }

    let matching_functions = find_named_functions(function_name.as_str())
        .flat_map(|fun| {
            FunctionMatch::create(fun, argument_types.as_slice())
        }).collect::<Vec<_>>();

    let best_match = matching_functions.iter().min_by_key(|fm| fm.distance());

    best_match.ok_or_else(|| {
        ResolveError
    }).and_then(|function_match| {
        // one last check to make sure that there isn't any ambiguity in the function call resolution
        let matching_count = matching_functions.iter().filter(|fm| fm.distance() == function_match.distance()).count();
        if matching_count > 1 {
            Err(ResolveError)
        } else {
            Ok(function_match.call.create(resolved_args))
        }
    })
}
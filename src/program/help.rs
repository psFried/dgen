use failure::Error;
use crate::interpreter::{Interpreter, Module};
use crate::program::DgenCommand;
use crate::{DataGenOutput, FunctionPrototype};


pub struct Help<'a> {
    module_name: Option<String>,
    function_name: Option<String>,
    interpreter: &'a mut Interpreter,
    print_source: bool,
}

impl<'a> Help<'a> {
    pub fn new(module_name: Option<String>, function_name: Option<String>, interpreter: &'a mut Interpreter, print_source: bool) -> Help {
        Help {
            module_name,
            function_name,
            interpreter,
            print_source,
        }
    }
}

impl<'a> DgenCommand for Help<'a> {
    fn execute(self, out: &mut DataGenOutput) -> Result<(), Error> {
        use std::fmt::Write;
        let Help {
            module_name,
            function_name,
            interpreter,
            print_source,
        } = self;

        match (module_name, function_name) {
            (Some(module), Some(function)) => {
                let iter = find_modules(&*interpreter, module.as_str())?;
                for actual_module in iter {
                    writeln!(out, "\nModule: {}", actual_module.name)?;
                    list_functions(
                        actual_module.function_iterator(),
                        Some(function.as_str()),
                        out,
                        print_source,
                    )?;
                }
            }
            (Some(module), None) => {
                let iter = find_modules(&interpreter, module.as_str())?;
                for actual_module in iter {
                    writeln!(out, "\nModule: {}", actual_module.name)?;
                    list_functions(actual_module.function_iterator(), None, out, print_source)?;
                }
            }
            (None, Some(function)) => {
                for module in interpreter.module_iterator() {
                    writeln!(out, "\nModule: {}", module.name)?;
                    list_functions(
                        module.function_iterator(),
                        Some(function.as_str()),
                        out,
                        print_source,
                    )?;
                }
            }
            _ => {
                // print some generic help and a listing of modules
                writeln!(out, "Available dgen modules: \n")?;
                for module in interpreter.module_iterator() {
                    writeln!(out, "{}", module.name)?;
                }
                writeln!(out, "\nTo list all the functions in a specific module, run `dgen help --module <name>`")?;
            }
        }
        Ok(())
    }
}

fn list_functions<'a, 'b, I: Iterator<Item = &'a FunctionPrototype>>(
    function_iterator: I,
    function_name: Option<&'b str>,
    out: &mut DataGenOutput,
    print_source: bool,
) -> Result<(), Error> {
    use std::fmt::Write;

    let mut filtered = function_iterator
        .filter(|fun| {
            function_name
                .as_ref()
                .map(|name| fun.name().contains(*name))
                .unwrap_or(true)
        }).peekable();

    if filtered.peek().is_none() {
        writeln!(out, "No matching functions")?;
    } else {
        writeln!(out, "")?;
        for fun in filtered {
            if print_source {
                writeln!(out, "{:#}", fun)?;
            } else {
                writeln!(out, "{}", fun)?;
            }
        }
    }
    Ok(())
}

fn has_matching_module(interpreter: &Interpreter, module_name: &str) -> Result<(), ()> {
    if interpreter
        .module_iterator()
        .any(|m| m.name.contains(module_name))
    {
        Ok(())
    } else {
        Err(())
    }
}

fn list_modules(interpreter: &Interpreter) -> String {
    use itertools::Itertools;
    interpreter
        .module_iterator()
        .map(|m| m.name.clone())
        .join("\n")
}

fn find_modules<'a>(
    interpreter: &'a Interpreter,
    module_name: &'a str,
) -> Result<impl Iterator<Item = &'a Module>, Error> {
    let _ = has_matching_module(interpreter, module_name).map_err(|_| {
        let other_modules = list_modules(interpreter);
        format_err!(
            "No module exists with name matching '{}'. Available modules are: \n\n{}\n",
            module_name,
            other_modules
        )
    })?;

    Ok(interpreter
        .module_iterator()
        .filter(move |m| m.name.contains(module_name)))
}

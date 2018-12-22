use crate::IString;
use crate::interpreter::prototype::{FunctionPrototype, BuiltinFunctionPrototype, InterpretedFunctionPrototype};
use crate::interpreter::ast::{WithSpan, MacroDef};
use crate::interpreter::{Source, UnreadSource};
use failure::Error;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct Module {
    pub name: IString,
    source: Arc<Source>,
    functions: HashMap<IString, Vec<FunctionPrototype>>,
}

impl Module {
    pub fn new(source: Arc<Source>, function_defs: Vec<WithSpan<MacroDef>>) -> Result<Module, Error> {
        let name = source.module_name();
        let mut module = Module {
            name,
            source: source.clone(),
            functions: HashMap::with_capacity(32)
        };

        for function in function_defs.into_iter() {
            let new_function = InterpretedFunctionPrototype::new(source.clone(), function);
            module.add_function(new_function.into())?;
        }

        Ok(module)
    }

    pub fn new_builtin<I: Iterator<Item = &'static BuiltinFunctionPrototype>>(builtin_fns: I) -> Module {
        use std::borrow::Cow;
        let mut module = Module {
            name: IString::from("dgen"),
            source: Arc::new(Source::new(UnreadSource::Builtin("dgen", ""), Cow::Borrowed(""))),
            functions: HashMap::with_capacity(64),
        };

        for fun in builtin_fns {
            module.add_function(FunctionPrototype::Builtin(fun)).expect("Failed to add builtin function to builtin module");
        }
        module
    }

    pub fn combine(&mut self, Module {functions, ..}: Module) -> Result<(), Error> {
        for (_, function_list) in functions.into_iter() {
            for new_function in function_list {
                self.add_function(new_function)?;
            }
        }
        Ok(())
    }

    pub fn add_function(&mut self, new_function: FunctionPrototype) -> Result<(), Error> {
        let Module { ref name, ref mut functions, .. } = *self;
        let new_function_name = new_function.name().into();
        let existing: &mut Vec<FunctionPrototype> = functions.entry(new_function_name).or_insert(Vec::new());

        if let Some(other_fun) = existing.iter().find(|existing| existing.is_same_signature(&new_function)) {
            bail!("Module '{}' contains multiple functions with the same signature. \nA: {} \nB: {}", name, other_fun, new_function);
        }
        existing.push(new_function);
        Ok(())
    }

    pub fn find_function(&self, name: &IString) -> Option<impl Iterator<Item = &FunctionPrototype>> {
        self.functions.get(name).map(|funs| funs.iter())
    }

    pub fn function_iterator(&self) -> impl Iterator<Item = &FunctionPrototype> {
        self.functions.values().flat_map(|funs| funs.iter())
    }
}
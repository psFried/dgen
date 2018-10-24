use IString;
use prototype::FunctionPrototype;
use interpreter::ast::MacroDef;
use failure::Error;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Module {
    pub name: IString,
    functions: HashMap<IString, Vec<FunctionPrototype>>,
}

impl Module {
    pub fn new(name: IString, function_defs: Vec<MacroDef>) -> Result<Module, Error> {
        let mut module = Module {
            name,
            functions: HashMap::with_capacity(32)
        };

        for function in function_defs.into_iter() {
            let new_function = FunctionPrototype::new(function);
            module.add_function(new_function)?;
        }

        Ok(module)
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
        let Module { ref name, ref mut functions } = *self;
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
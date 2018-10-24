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
        let mut functions = HashMap::with_capacity(32);

        for function in function_defs.into_iter() {
            let new_function_name = function.name.clone();
            let new_function = FunctionPrototype::new(function);
            
            let mut existing: &mut Vec<FunctionPrototype> = functions.entry(new_function_name).or_insert(Vec::new());

            if let Some(other_fun) = existing.iter().find(|existing| existing.is_same_signature(&new_function)) {
                bail!("Module '{}' contains multiple functions with the same signature. \nA: {} \nB: {}", name, other_fun, new_function);
            }
            existing.push(new_function);
        }

        Ok(Module {
            name, functions
        })
    }

    pub fn find_function(&self, name: &IString) -> Option<impl Iterator<Item = &FunctionPrototype>> {
        self.functions.get(name).map(|funs| funs.iter())
    }

    pub fn function_iterator(&self) -> impl Iterator<Item = &FunctionPrototype> {
        self.functions.values().flat_map(|funs| funs.iter())
    }
}
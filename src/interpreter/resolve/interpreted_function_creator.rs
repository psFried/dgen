use failure::Error;
use generator::{GeneratorArg, GeneratorType};
use interpreter::ast::MacroDef;
use interpreter::functions::FunctionCreator;
use interpreter::ProgramContext;

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
        macro_def.doc_comments = Vec::new(); // just to deallocate the memory
        let arg_types = macro_def.args.iter().map(|a| a.arg_type).collect();
        MacroDefFunctionCreator {
            macro_def,
            arg_types,
            description,
        }
    }

    pub fn bind_arguments(&self, args: Vec<GeneratorArg>) -> Vec<MacroArgFunctionCreator> {
        args.into_iter()
            .zip(self.macro_def.args.iter())
            .map(|(value, arg_type)| MacroArgFunctionCreator::new(arg_type.name.clone(), value))
            .collect()
    }
}

pub struct MacroArgFunctionCreator {
    name: String,
    value: GeneratorArg,
}

impl MacroArgFunctionCreator {
    pub fn new(name: String, value: GeneratorArg) -> MacroArgFunctionCreator {
        MacroArgFunctionCreator { name, value }
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
    fn create(
        &self,
        _args: Vec<GeneratorArg>,
        _ctx: &ProgramContext,
    ) -> Result<GeneratorArg, Error> {
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

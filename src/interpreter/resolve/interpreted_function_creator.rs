use failure::Error;
use generator::GeneratorArg;
use interpreter::ast::{Expr, MacroDef};
use interpreter::functions::{FunctionArgs, FunctionCreator, EMPTY_ARGS};
use interpreter::ProgramContext;
use IString;

pub struct MacroDefFunctionCreator {
    description: String, // no point in interning the doc comments, since they're not likely to be repeated or compared
    function_name: IString,
    args: FunctionArgs,
    body: Expr,
}

impl MacroDefFunctionCreator {
    pub fn new(macro_def: MacroDef) -> MacroDefFunctionCreator {
        let MacroDef {
            name,
            args,
            body,
            doc_comments,
        } = macro_def;

        let args = args.into_iter().collect();
        MacroDefFunctionCreator {
            description: doc_comments,
            args,
            body,
            function_name: name,
        }
    }

    pub fn bind_arguments(&self, args: Vec<GeneratorArg>) -> Vec<MacroArgFunctionCreator> {
        args.into_iter()
            .zip(self.args.arg_types.iter())
            .map(|(value, arg)| MacroArgFunctionCreator::new(arg.name.clone(), value))
            .collect()
    }
}

pub struct MacroArgFunctionCreator {
    name: IString,
    value: GeneratorArg,
}

impl MacroArgFunctionCreator {
    pub fn new(name: IString, value: GeneratorArg) -> MacroArgFunctionCreator {
        MacroArgFunctionCreator { name, value }
    }
}

impl FunctionCreator for MacroArgFunctionCreator {
    fn get_name(&self) -> IString {
        self.name.clone()
    }
    fn get_arg_types(&self) -> &FunctionArgs {
        &EMPTY_ARGS
    }
    fn get_description(&self) -> &str {
        &*self.name
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
    fn get_name(&self) -> IString {
        self.function_name.clone()
    }

    fn get_arg_types(&self) -> &FunctionArgs {
        &self.args
    }

    fn get_description(&self) -> &str {
        self.description.as_str()
    }

    fn create(&self, args: Vec<GeneratorArg>, ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        let bound_args = self.bind_arguments(args);
        ctx.resolve_macro_call(&self.body, bound_args)
    }
}

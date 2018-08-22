use failure::Error;
use generator::{self, GeneratorArg, GeneratorType};
use interpreter::ast;
use interpreter::resolve::ProgramContext;
use regex::Regex;
use std::iter::FromIterator;
use IString;

#[derive(Clone, PartialEq, Debug)]
pub struct FunctionArg {
    pub name: IString,
    pub arg_type: GeneratorType,
}

impl FunctionArg {
    pub fn new<S: Into<IString>>(name: S, arg_type: GeneratorType) -> FunctionArg {
        FunctionArg {
            name: name.into(),
            arg_type,
        }
    }
}

pub struct FunctionArgs {
    pub arg_types: Vec<FunctionArg>,
    pub last_arg_variadic: bool,
}

impl FunctionArgs {
    pub fn empty() -> FunctionArgs {
        FunctionArgs {
            arg_types: Vec::new(),
            last_arg_variadic: false,
        }
    }
}

impl FromIterator<ast::MacroArgument> for FunctionArgs {
    fn from_iter<I: IntoIterator<Item = ast::MacroArgument>>(iter: I) -> Self {
        let mut args = Vec::new();
        for arg in iter {
            let ast::MacroArgument { name, arg_type } = arg;
            args.push(FunctionArg::new(name, arg_type));
        }
        FunctionArgs {
            arg_types: args,
            last_arg_variadic: false,
        }
    }
}

pub struct ArgsBuilder(FunctionArgs);

impl ArgsBuilder {
    pub fn new() -> ArgsBuilder {
        ArgsBuilder(FunctionArgs::empty())
    }

    pub fn arg(mut self, name: &'static str, arg_type: GeneratorType) -> ArgsBuilder {
        self.0.arg_types.push(FunctionArg::new(name, arg_type));
        self
    }

    pub fn variadic(mut self) -> FunctionArgs {
        self.0.last_arg_variadic = true;
        self.build()
    }

    pub fn build(self) -> FunctionArgs {
        self.0
    }
}

impl Into<FunctionArgs> for ArgsBuilder {
    fn into(self) -> FunctionArgs {
        self.build()
    }
}

pub trait FunctionCreator: 'static {
    fn get_name(&self) -> IString;
    fn get_arg_types(&self) -> &FunctionArgs;
    fn get_description(&self) -> &str;
    fn create(&self, args: Vec<GeneratorArg>, ctx: &ProgramContext) -> Result<GeneratorArg, Error>;
}

lazy_static!{
    pub static ref EMPTY_ARGS: FunctionArgs = FunctionArgs {
        arg_types: Vec::new(),
        last_arg_variadic: false,
    };
    pub static ref BUILTIN_FUNCTIONS: Vec<BuiltinFunctionCreator> = { vec![
        // char generators
        generator::chars::alphanumeric_builtin(),
        generator::chars::unicode_bmp_builtin(),
        generator::chars::unicode_scalar_builtin(),
        // string generator
        generator::string::string_builtin(),
        // unsigned integers
        generator::uint::max_uint_builtin(),
        generator::uint::uint2_builtin(),
        // Signed integers
        generator::int::signed_int_fun2_creator(),
        generator::int::signed_int_max(),
        generator::int::signed_int_min(),
        // various generators that select from among their arguments
        generator::one_of::one_of_fun(),
        generator::either::either_freq_fun(),
        generator::either::either_fun(),
        generator::stable_select::stable_select_fun(),
        // generators that compose their arguments
        generator::concat::concat_delimited_function_creator(),
        generator::concat::simple_concat_function_creator(),
        generator::repeat::repeat_delimited_fun(),
        generator::repeat::repeat_fun(),
        // selecting from files
        generator::file::select_from_file_fun(),
        generator::file::words_fun(),
    ] };
}

pub struct BuiltinFunctionCreator {
    pub name: IString,
    pub description: &'static str,
    pub args: FunctionArgs,
    pub create_fn:
        &'static (Fn(Vec<GeneratorArg>, &ProgramContext) -> Result<GeneratorArg, Error> + Sync),
}

impl BuiltinFunctionCreator {
    pub fn new(
        name: &'static str,
        description: &'static str,
        args: FunctionArgs,
        create_fn: &'static (Fn(Vec<GeneratorArg>, &ProgramContext) -> Result<GeneratorArg, Error>
                      + Sync),
    ) -> BuiltinFunctionCreator {
        BuiltinFunctionCreator {
            name: name.into(),
            description,
            args,
            create_fn,
        }
    }
}

impl FunctionCreator for BuiltinFunctionCreator {
    fn get_name(&self) -> IString {
        self.name.clone()
    }

    fn get_description(&self) -> &str {
        self.description
    }

    fn get_arg_types(&self) -> &FunctionArgs {
        &self.args
    }

    fn create(&self, args: Vec<GeneratorArg>, ctx: &ProgramContext) -> Result<GeneratorArg, Error> {
        (self.create_fn)(args, ctx)
    }
}

pub enum FunctionNameFilter {
    ExactMatch(IString),
    Regex(Regex),
    All,
}
impl FunctionNameFilter {
    pub fn matches(&self, function_name: IString) -> bool {
        match *self {
            FunctionNameFilter::ExactMatch(ref name) => *name == function_name,
            FunctionNameFilter::Regex(ref regex) => regex.is_match(&*function_name),
            _ => true,
        }
    }
}

pub struct BuiltinFunctionIterator {
    idx: usize,
    filter: FunctionNameFilter,
}

impl BuiltinFunctionIterator {
    pub fn new(filter: FunctionNameFilter) -> BuiltinFunctionIterator {
        BuiltinFunctionIterator { idx: 0, filter }
    }
}

impl Iterator for BuiltinFunctionIterator {
    type Item = &'static FunctionCreator;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current_index = self.idx;
            let result = BUILTIN_FUNCTIONS.get(current_index);
            self.idx += 1;

            if let Some(bc) = result {
                if self.filter.matches(bc.get_name()) {
                    return Some(bc as &FunctionCreator);
                } // else loop around for another try
            } else {
                return None;
            }
        }
    }
}
pub struct FunctionHelp<'a>(pub &'a FunctionCreator);

use std::fmt;
impl<'a> fmt::Display for FunctionHelp<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.0.get_name())?;

        let mut first = true;
        for arg in self.0.get_arg_types().arg_types.iter() {
            if !first {
                f.write_str(", ")?;
            } else {
                first = false;
            }
            write!(f, "{}: {}", arg.name, arg.arg_type)?;
        }

        write!(f, ") - {}", self.0.get_description())
    }
}

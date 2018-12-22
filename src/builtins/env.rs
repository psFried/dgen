use failure::Error;

use std::collections::HashMap;
use std::rc::Rc;
use IString;
use {
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput,
    DynStringFun, GenType, ProgramContext, RunnableFunction,
};

fn create_env_map() -> HashMap<IString, IString> {
    let mut map = HashMap::with_capacity(64);
    for (key, value) in ::std::env::vars() {
        map.insert(key.into(), value.into());
    }
    map
}

lazy_static! {
    static ref ENV_VARS: HashMap<IString, IString> = create_env_map();
}

#[derive(Debug)]
struct EnvVar {
    key: DynStringFun,
}

impl RunnableFunction<IString> for EnvVar {
    fn gen_value(&self, context: &mut ProgramContext) -> Result<IString, Error> {
        let key = self.key.gen_value(context)?;
        ENV_VARS
            .get(&key)
            .cloned()
            .ok_or_else(|| format_err!("No such env variable: '{}'", key))
    }
    fn write_value(
        &self,
        context: &mut ProgramContext,
        out: &mut DataGenOutput,
    ) -> Result<(), Error> {
        let value = self.gen_value(context)?;
        out.write(&value)
    }
}

const ARG_NAME: &str = "env_var_name";

fn create_env(args: Arguments) -> CreateFunctionResult {
    let key = args.required_arg(ARG_NAME, 0, AnyFunction::require_string)?;
    Ok(AnyFunction::String(Rc::new(EnvVar { key })))
}

pub const ENV_VAR: &BuiltinFunctionPrototype = &BuiltinFunctionPrototype {
    function_name: "env",
    description:
        "Returns the value of the given env variable, throws an error if the env var is not set",
    arguments: &[(ARG_NAME, GenType::String)],
    variadic: false,
    create_fn: &create_env,
};

#[cfg(test)]
mod test {
    use fun_test::test_program_success;

    #[test]
    fn returns_the_value_of_an_environment_variable() {
        let (key, value) = ::std::env::vars()
            .next()
            .expect("At least one environment variable has to be set");
        let program = format!(r#"env("{}")"#, key);
        test_program_success(1, program.as_str(), value.as_str());
    }
}

use failure::Error;
use crate::interpreter::SourceRef;
use std::fmt::Debug;
use std::rc::Rc;
use crate::{AnyFunction, DataGenOutput, DynFun, IString, ProgramContext, RunnableFunction};

#[derive(Debug)]
pub struct RuntimeWrapper<T: Debug + 'static> {
    wrapped: DynFun<T>,
    function_name: IString,
    source_ref: SourceRef,
}

pub fn wrap(
    any_function: AnyFunction,
    function_name: IString,
    source_ref: SourceRef,
) -> AnyFunction {
    match any_function {
        AnyFunction::Boolean(fun) => {
            AnyFunction::Boolean(RuntimeWrapper::new(fun, function_name, source_ref))
        }
        AnyFunction::Bin(fun) => {
            AnyFunction::Bin(RuntimeWrapper::new(fun, function_name, source_ref))
        }
        AnyFunction::String(fun) => {
            AnyFunction::String(RuntimeWrapper::new(fun, function_name, source_ref))
        }
        AnyFunction::Uint(fun) => {
            AnyFunction::Uint(RuntimeWrapper::new(fun, function_name, source_ref))
        }
        AnyFunction::Int(fun) => {
            AnyFunction::Int(RuntimeWrapper::new(fun, function_name, source_ref))
        }
        AnyFunction::Decimal(fun) => {
            AnyFunction::Decimal(RuntimeWrapper::new(fun, function_name, source_ref))
        }
    }
}

impl<T: Debug + 'static> RuntimeWrapper<T> {
    fn new(wrapped: DynFun<T>, function_name: IString, source_ref: SourceRef) -> DynFun<T> {
        Rc::new(RuntimeWrapper {
            wrapped,
            function_name,
            source_ref,
        })
    }

    fn handle_error<S>(
        &self,
        result: Result<S, Error>,
        context: &mut ProgramContext,
    ) -> Result<S, Error> {
        if let Some(err) = result.as_ref().err() {
            context.error(&self.function_name, &self.source_ref, err);
        }
        result
    }
}

impl<T: Debug + 'static> RunnableFunction<T> for RuntimeWrapper<T> {
    fn gen_value(&self, context: &mut ProgramContext) -> Result<T, Error> {
        let result = self.wrapped.gen_value(context);
        self.handle_error(result, context)
    }

    fn write_value(
        &self,
        context: &mut ProgramContext,
        out: &mut DataGenOutput,
    ) -> Result<(), Error> {
        let result = self.wrapped.write_value(context, out);
        self.handle_error(result, context)
    }
}

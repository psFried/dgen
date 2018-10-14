
use v2::{GenType, AnyFunction};
use failure::Error;
use std::ops::Range;

pub struct Arguments(Vec<AnyFunction>);

impl Arguments {
    pub fn new(args: Vec<AnyFunction>) -> Arguments {
        Arguments(args)
    }

    pub fn get_arg_type(&self, index: usize) -> Option<GenType> {
        self.0.get(index).map(|a| a.get_type())
    }

    pub fn required_arg<F, R>(&self, name: &str, position: usize, convert_fun: F) -> Result<R, Error> 
    where F: Fn(AnyFunction) -> Result<R, Error> {
        if let Some(any) = self.0.get(position) {
            convert_fun(any.clone())
        } else {
            bail!("No argument provided for arg '{}' at position {}", name, position)
        }
    }

    fn get_optional_varargs<F, R>(&self, start_position: usize, convert: F) -> Result<Vec<R>, Error>
    where  F: Fn(AnyFunction) -> Result<R, Error> {
        self.0.iter().skip(start_position).cloned().map(convert).collect()
    }

    pub fn get_required_varargs<F, R>(&self, name: &str, start_position: usize, convert: F) -> Result<Vec<R>, Error>
    where  F: Fn(AnyFunction) -> Result<R, Error> {
        self.get_optional_varargs(start_position, convert).and_then(|args| {
            if args.is_empty() {
                Err(format_err!("Missing required varargs '{}' starting at position {}. At least one argument is required ", name, start_position))
            } else {
                Ok(args)
            }
        })
    }

    pub fn require_2_args<F1, R1, F2, R2>(
        self,
        arg1_name: &'static str,
        af1: F1,
        arg2_name: &'static str,
        af2: F2,
    ) -> Result<(R1, R2), Error>
    where
        F1: Fn(AnyFunction) -> Result<R1, Error>,
        F2: Fn(AnyFunction) -> Result<R2, Error>,
    {
        let r1 = self.required_arg(arg1_name, 0, af1)?;
        let r2 = self.required_arg(arg2_name, 1, af2)?;
        Ok((r1, r2))
    }

    pub fn require_3_args<F1, R1, F2, R2, F3, R3>(
        self,
        arg1_name: &'static str,
        af1: F1,
        arg2_name: &'static str,
        af2: F2,
        arg3_name: &'static str,
        af3: F3,
    ) -> Result<(R1, R2, R3), Error>
    where
        F1: Fn(AnyFunction) -> Result<R1, Error>,
        F2: Fn(AnyFunction) -> Result<R2, Error>,
        F3: Fn(AnyFunction) -> Result<R3, Error>,
    {
        let r1 = self.required_arg(arg1_name, 0, af1)?;
        let r2 = self.required_arg(arg2_name, 1, af2)?;
        let r3 = self.required_arg(arg3_name, 2, af3)?;
        Ok((r1, r2, r3))
    }
}


use super::FunctionCreator;
use generator::{GeneratorArg, GeneratorType};
use generator::one_of::OneOfGenerator;


// TODO: Add OneOf_ functions for other primitive types
pub struct OneOfUint;
impl FunctionCreator for OneOfUint {
    fn get_name(&self) -> &'static str {
        "oneOf"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[GeneratorType::UnsignedInt], true)
    }

    fn get_description(&self) -> &'static str {
        "randomly selects one of the given arguments using a uniform distribution"
    }

    fn create(&self, args: Vec<GeneratorArg>) -> GeneratorArg {
        create_one_of(args)
    }
}

pub struct OneOfString;
impl FunctionCreator for OneOfString {
    fn get_name(&self) -> &'static str {
        "oneOf"
    }

    fn get_arg_types(&self) -> (&'static [GeneratorType], bool) {
        (&[GeneratorType::String], true)
    }

    fn get_description(&self) -> &'static str {
        "randomly selects one of the given arguments using a uniform distribution. Allows for mixed input types"
    }

    fn create(&self, args: Vec<GeneratorArg>) -> GeneratorArg {
        create_one_of(args)
    }
}


fn create_one_of(args: Vec<GeneratorArg>) -> GeneratorArg {
        let initial_type = args[0].get_type();
        let target_type = args.iter().fold(initial_type, |target_type, arg| {
            let arg_type = arg.get_type();
            if arg_type == target_type {
                target_type
            } else {
                GeneratorType::String
            }
        });

        match target_type {
            GeneratorType::UnsignedInt => {
                let generators = args.into_iter().map(|a| a.as_uint().unwrap()).collect::<Vec<_>>();
                GeneratorArg::UnsignedInt(OneOfGenerator::new(generators))
            }
            GeneratorType::String => {
                let generators = args.into_iter().map(|a| a.as_string()).collect::<Vec<_>>();
                GeneratorArg::String(OneOfGenerator::new(generators))
            }
            _ => {
                unimplemented!();
            }
        }

}
mod nullable;
mod string;
mod one_of;
mod constant;

pub use self::nullable::NullableGenerator;
pub use self::string::{StringGenerator, AsciiChar};
pub use self::one_of::OneOfGenerator;
pub use self::constant::ConstantGenerator;

use formatter::Formatter;
use std::fmt::{self, Display, Debug, Write};
use std::io;
use std::str::FromStr;
use rand::Rng;
use std::marker::PhantomData;

pub type DataGenRng = ::rand::prng::Hc128Rng;

pub type DynCharGenerator = Box<Generator<Output=char>>;
pub type DynDecimalGenerator = Box<Generator<Output=f64>>;
pub type DynUnsignedIntGenerator = Box<Generator<Output=u64>>;
pub type DynSignedIntGenerator = Box<Generator<Output=i64>>;
pub type DynStringGenerator = Box<Generator<Output=String>>;

pub enum GeneratorArg {
    Char(DynCharGenerator),
    Decimal(DynDecimalGenerator),
    UnsignedInt(DynUnsignedIntGenerator),
    SignedInt(DynSignedIntGenerator),
    String(DynStringGenerator),
}

impl GeneratorArg {
    fn get_type(&self) -> GeneratorType {
        match *self {
            GeneratorArg::Char(_) => GeneratorType::Char,
            GeneratorArg::Decimal(_) => GeneratorType::Decimal,
            GeneratorArg::UnsignedInt(_) => GeneratorType::UnsignedInt,
            GeneratorArg::SignedInt(_) => GeneratorType::SignedInt,
            GeneratorArg::String(_) => GeneratorType::String,
        }
    }

}



#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GeneratorType {
    UnsignedInt,
    SignedInt,
    Decimal,
    Char,
    String,
}

impl Display for GeneratorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let stringy  = match *self {
            GeneratorType::Char => "Char",
            GeneratorType::Decimal => "Float",
            GeneratorType::UnsignedInt => "UnsignedInt",
            GeneratorType::SignedInt => "SignedInt",
            GeneratorType::String => "String",
        };
        f.write_str(stringy)
    }
}

impl Display for GeneratorArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = self.get_type();
        match *self {
            GeneratorArg::Char(ref gen) => {
                write!(f, "{}: {}", gen, description)
            },
            GeneratorArg::Decimal(ref gen) => {
                write!(f, "{}: {}", gen, description)
            },
            GeneratorArg::UnsignedInt(ref gen) => {
                write!(f, "{}: {}", gen, description)
            },
            GeneratorArg::SignedInt(ref gen) => {
                write!(f, "{}: {}", gen, description)
            },
            GeneratorArg::String(ref gen) => {
                write!(f, "{}: {}", gen, description)
            },
        }
    }
}

pub trait Generator: Display {
    type Output: Display;
    fn gen_value(&mut self, rng: &mut DataGenRng) -> Option<&Self::Output>;


}


pub trait BoxedGen {
    fn gen_displayable(&mut self, rng: &mut DataGenRng) -> Option<&Display>;
}

impl <G> BoxedGen for G where G: Generator, G::Output: Display + 'static {
    fn gen_displayable(&mut self, rng: &mut DataGenRng) -> Option<&Display> {
        self.gen_value(rng).map(|v| v as &Display)
    }
}



pub struct Column {
    name: String,
    generator: Box<BoxedGen>,
}

impl Column {
    pub fn new(name: String, generator: Box<BoxedGen>) -> Column {
        Column {
            name,
            generator,
        }
    }

    pub fn write_column<R: Rng, F: Formatter>(&mut self, rng: &mut DataGenRng, formatter: &mut F) -> io::Result<()> {
        formatter.write_column_start(self.name()).and_then(|()| {
            if let Some(display) = self.generator.gen_displayable(rng) {
                formatter.write_value(display)
            } else {
                formatter.write_null()
            }
        }).and_then(|()| {
            formatter.write_column_end(self.name())
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

}

pub struct InvalidGeneratorArguments {
    args: Vec<GeneratorArg>,
    desc: &'static str,
}

impl InvalidGeneratorArguments {
    fn new(desc: &'static str, args: Vec<GeneratorArg>) -> InvalidGeneratorArguments {
        InvalidGeneratorArguments {desc, args}
    }
}

impl Display for InvalidGeneratorArguments {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}





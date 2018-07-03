pub mod nullable;
pub mod string;
pub mod one_of;
pub mod constant;

use std::fmt::{self, Display};

pub type DataGenRng = ::rand::prng::XorShiftRng;

pub type DynCharGenerator = Box<Generator<Output=char>>;
pub type DynDecimalGenerator = Box<Generator<Output=f64>>;
pub type DynUnsignedIntGenerator = Box<Generator<Output=u64>>;
pub type DynSignedIntGenerator = Box<Generator<Output=i64>>;
pub type DynStringGenerator = Box<Generator<Output=String>>;
pub type DynAnyGenerator = Box<Generator<Output=Box<Display>>>;

pub enum GeneratorArg {
    Char(DynCharGenerator),
    Decimal(DynDecimalGenerator),
    UnsignedInt(DynUnsignedIntGenerator),
    SignedInt(DynSignedIntGenerator),
    String(DynStringGenerator),
    Any(DynAnyGenerator),
}

impl GeneratorArg {
    pub fn get_type(&self) -> GeneratorType {
        match *self {
            GeneratorArg::Char(_) => GeneratorType::Char,
            GeneratorArg::Decimal(_) => GeneratorType::Decimal,
            GeneratorArg::UnsignedInt(_) => GeneratorType::UnsignedInt,
            GeneratorArg::SignedInt(_) => GeneratorType::SignedInt,
            GeneratorArg::String(_) => GeneratorType::String,
            GeneratorArg::Any(_) => GeneratorType::Any,
        }
    }

    pub fn as_uint(self) -> Option<DynUnsignedIntGenerator> {
        match self {
            GeneratorArg::UnsignedInt(gen) => Some(gen),
            _ => None
        }
    }
    
    pub fn gen_displayable(&mut self, rng: &mut DataGenRng) -> Option<&Display> {
        match self {
            GeneratorArg::Char(ref mut gen) => gen.gen_value(rng).map(|v| v as &Display),
            GeneratorArg::Decimal(ref mut gen) => gen.gen_value(rng).map(|v| v as &Display),
            GeneratorArg::UnsignedInt(gen) => gen.gen_value(rng).map(|v| v as &Display),
            GeneratorArg::SignedInt(gen) => gen.gen_value(rng).map(|v| v as &Display),
            GeneratorArg::String(gen) => gen.gen_value(rng).map(|v| v as &Display),
            GeneratorArg::Any(gen) => gen.gen_value(rng).map(|v| v as &Display),
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
    Any,
}

impl Display for GeneratorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let stringy  = match *self {
            GeneratorType::Char => "Char",
            GeneratorType::Decimal => "Float",
            GeneratorType::UnsignedInt => "UnsignedInt",
            GeneratorType::SignedInt => "SignedInt",
            GeneratorType::String => "String",
            GeneratorType::Any => "Any",
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
            GeneratorArg::Any(ref gen) => {
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


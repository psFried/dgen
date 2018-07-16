pub mod either;
pub mod nullable;
pub mod string;
pub mod one_of;
pub mod constant;
pub mod uint;
pub mod concat;
pub mod repeat;

use std::fmt::{self, Display};
use std::io;
use std::clone::Clone;
use writer::DataGenOutput;

pub type DataGenRng = ::rand::prng::XorShiftRng;

pub type DynGenerator<T> = Box<Generator<Output=T>>;

pub type DynBoolGenerator = DynGenerator<bool>;
pub type DynCharGenerator = DynGenerator<char>;
pub type DynDecimalGenerator = DynGenerator<f64>;
pub type DynUnsignedIntGenerator = DynGenerator<u64>;
pub type DynSignedIntGenerator = DynGenerator<i64>;
pub type DynStringGenerator = DynGenerator<String>;

#[allow(unused)]
pub enum GeneratorArg {
    Bool(DynBoolGenerator),
    Char(DynCharGenerator),
    Decimal(DynDecimalGenerator),
    UnsignedInt(DynUnsignedIntGenerator),
    SignedInt(DynSignedIntGenerator),
    String(DynStringGenerator),
}

impl fmt::Debug for GeneratorArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GeneratorArg({})", self.get_type())
    }
}

impl GeneratorArg {
    pub fn get_type(&self) -> GeneratorType {
        match *self {
            GeneratorArg::Bool(_) => GeneratorType::Boolean,
            GeneratorArg::Char(_) => GeneratorType::Char,
            GeneratorArg::Decimal(_) => GeneratorType::Decimal,
            GeneratorArg::UnsignedInt(_) => GeneratorType::UnsignedInt,
            GeneratorArg::SignedInt(_) => GeneratorType::SignedInt,
            GeneratorArg::String(_) => GeneratorType::String,
        }
    }

    pub fn as_uint(self) -> Option<DynUnsignedIntGenerator> {
        match self {
            GeneratorArg::UnsignedInt(gen) => Some(gen),
            _ => None
        }
    }

    pub fn as_bool(self) -> Option<DynBoolGenerator> {
        match self {
            GeneratorArg::Bool(gen) => Some(gen),
            _ => None
        }
    }

    pub fn as_decimal(self) -> Option<DynDecimalGenerator> {
        match self {
            GeneratorArg::Decimal(gen) => Some(gen),
            _ => None
        }
    }

    pub fn as_char(self) -> Option<DynCharGenerator> {
        match self {
            GeneratorArg::Char(gen) => Some(gen),
            _ => None
        }
    }

    pub fn as_signed_int(self) -> Option<DynSignedIntGenerator> {
        match self {
            GeneratorArg::SignedInt(gen) => Some(gen),
            _ => None
        }
    }

    pub fn as_string(self) -> DynStringGenerator {
        match self {
            GeneratorArg::Bool(gen) => WrappedAnyGen::new(gen),
            GeneratorArg::Char(gen) => WrappedAnyGen::new(gen),
            GeneratorArg::Decimal(gen) => WrappedAnyGen::new(gen),
            GeneratorArg::UnsignedInt(gen) => WrappedAnyGen::new(gen),
            GeneratorArg::SignedInt(gen) => WrappedAnyGen::new(gen),
            GeneratorArg::String(gen) => gen,
        }
    }
    
    pub fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> io::Result<u64> {
        match self {
            GeneratorArg::Bool(gen) => gen.write_value(rng, output),
            GeneratorArg::Char(gen) => gen.write_value(rng, output),
            GeneratorArg::Decimal(gen) => gen.write_value(rng, output),
            GeneratorArg::UnsignedInt(gen) => gen.write_value(rng, output),
            GeneratorArg::SignedInt(gen) => gen.write_value(rng, output),
            GeneratorArg::String(gen) => gen.write_value(rng, output),
        }
    }
}

impl Clone for GeneratorArg {
    fn clone(&self) -> GeneratorArg {
        match self {
            GeneratorArg::Bool(gen) => GeneratorArg::Bool(gen.new_from_prototype()),
            GeneratorArg::Char(gen) => GeneratorArg::Char(gen.new_from_prototype()),
            GeneratorArg::Decimal(gen) => GeneratorArg::Decimal(gen.new_from_prototype()),
            GeneratorArg::UnsignedInt(gen) => GeneratorArg::UnsignedInt(gen.new_from_prototype()),
            GeneratorArg::SignedInt(gen) => GeneratorArg::SignedInt(gen.new_from_prototype()),
            GeneratorArg::String(gen) => GeneratorArg::String(gen.new_from_prototype()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GeneratorType {
    Boolean,
    UnsignedInt,
    SignedInt,
    Decimal,
    Char,
    String,
}

impl Display for GeneratorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let stringy  = match *self {
            GeneratorType::Boolean => "Boolean",
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
            GeneratorArg::Bool(ref gen) => {
                write!(f, "{}: {}", gen, description)
            },
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



pub trait Generator: Display + Send {
    type Output: Display;
    fn gen_value(&mut self, rng: &mut DataGenRng) -> Option<&Self::Output>;

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> io::Result<u64>;

    fn new_from_prototype(&self) -> Box<Generator<Output=Self::Output>>;
}


pub struct WrappedAnyGen<T: Display> {
    wrapped: Box<Generator<Output=T>>,
    buf: String,
}

impl <T: Display + 'static> WrappedAnyGen<T> {
    pub fn new(gen: Box<Generator<Output=T>>) -> DynStringGenerator {
        Box::new(WrappedAnyGen {
            wrapped: gen,
            buf: String::with_capacity(32)
        })
    }
}

impl <T: Display + 'static> Generator for WrappedAnyGen<T> {
    type Output = String;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Option<&String> {
        use std::fmt::Write;
        let WrappedAnyGen {ref mut wrapped, ref mut buf} = *self;
        buf.clear();

        wrapped.gen_value(rng).map(move |t|  {
            // this isn't something that can practically fail at runtime since there's no io involved
            let _ = buf.write_fmt(format_args!("{}", t));
            &*buf
        })
    }

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> io::Result<u64> {
        if let Some(value) = self.gen_value(rng) {
            output.write_string(value)
        } else {
            Ok(0)
        }
    }
    
    fn new_from_prototype(&self) -> Box<Generator<Output=String>> {
        let wrapped = self.wrapped.new_from_prototype();
        let buf = String::with_capacity(self.buf.capacity());
        Box::new(WrappedAnyGen { wrapped, buf })
    }
}

impl <T: Display + 'static> Display for WrappedAnyGen<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.wrapped)
    }
}
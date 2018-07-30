use interpreter::Interpreter;
use generator::DataGenRng;
use writer::DataGenOutput;
use failure::Error;

use std::path::PathBuf;
use std::io::{self, Read};
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// reference to a file on the local filesystem
    File(PathBuf),
    /// source is held entirely in memory
    String(String),
    /// used for the standard lib, which is included in the binary
    Builtin(&'static str),
    /// source will be read dynamically from stdin. It is an error to have more than one sourceType that uses stdin
    Stdin
}

impl Source {
    pub fn file<P: Into<PathBuf>>(path: P) -> Source {
        Source::File(path.into())
    }
    pub fn string<S: Into<String>>(string: S) -> Source {
        Source::String(string.into())
    }
    pub fn stdin() -> Source {
        Source::Stdin
    }

    pub fn read_to_str<'a>(&'a self) -> Result<Cow<'a, str>, Error> {
        match *self {
            Source::File(ref path) => {
                use std::fs::File;
                let mut file = File::open(path)?;
                let mut buffer = String::with_capacity(512);
                file.read_to_string(&mut buffer)?;
                Ok(buffer.into())
            }
            Source::String(ref string) => Ok(string.as_str().into()),
            Source::Builtin(ref builtin) => Ok((*builtin).into()),
            Source::Stdin => {
                let mut sin = io::stdin();
                let mut buffer = String::with_capacity(512);
                sin.read_to_string(&mut buffer)?;
                Ok(buffer.into())
            }
        }
    }
}

impl From<String> for Source {
    fn from(s: String) -> Source {
        Source::string(s)
    }
}
impl From<&'static str> for Source {
    fn from(s: &'static str) -> Source {
        Source::Builtin(s)
    }
}

use std::path::Path;
impl <'a> From<&'a Path> for Source {
    fn from(p: &'a Path) -> Source {
        Source::file(p)
    }
}

impl From<PathBuf> for Source {
    fn from(p: PathBuf) -> Source {
        Source::file(p)
    }
}

pub struct Program {
    iterations: u64,
    source: Source,
    rng: DataGenRng,
    interpreter: Interpreter,
}

impl Program {

    pub fn new<T: Into<Source>>(
        verbosity: u64,
        iterations: u64,
        source: T,
        rng: DataGenRng,
    ) -> Program {
        Program {
            iterations,
            source: source.into(),
            rng,
            interpreter: Interpreter::new(verbosity),
        }
    }

    pub fn run(self, output: &mut DataGenOutput) -> Result<(), Error> {
        let Program {
            iterations,
            source,
            mut rng,
            mut interpreter,
            ..
        } = self;

        let src_string = source.read_to_str()?;
        let mut generator = interpreter.eval_program(src_string.as_ref())?;

        for _ in 0..iterations {
            generator.write_value(&mut rng, output)?;
        }
        output.flush().map_err(Into::into)
    }

    pub fn add_library<T: Into<Source>>(&mut self, lib_source: T) -> Result<(), Error> {
        let source = lib_source.into();
        let as_str = source.read_to_str()?;
        self.interpreter.eval_library(as_str.as_ref())
    }
}

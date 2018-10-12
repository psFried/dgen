use failure::Error;
use generator::DataGenRng;
use writer::DataGenOutput;

use std::borrow::Cow;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// reference to a file on the local filesystem
    File(PathBuf),
    /// source is held entirely in memory
    String(String),
    /// used for the standard lib, which is included in the binary
    Builtin(&'static str),
    /// source will be read dynamically from stdin. It is an error to have more than one sourceType that uses stdin
    Stdin,
}

impl Source {
    pub fn get_name(&self) -> String {
        match *self {
            Source::File(ref pb) => pb
                .file_name()
                .map(|name| {
                    name.to_str()
                        .map(::std::borrow::ToOwned::to_owned)
                        .unwrap_or_else(|| "unknown file".to_owned())
                }).unwrap_or_else(|| "unknown file".to_owned()),
            Source::String(_) => "string input".to_owned(),
            Source::Builtin(ref name) => (*name).to_owned(),
            Source::Stdin => "stdin".to_owned(),
        }
    }
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
impl<'a> From<&'a Path> for Source {
    fn from(p: &'a Path) -> Source {
        Source::file(p)
    }
}

impl From<PathBuf> for Source {
    fn from(p: PathBuf) -> Source {
        Source::file(p)
    }
}

enum InterpreterType {
    V1(::interpreter::Interpreter),
    V2(::v2::interpreter::Interpreter),
}

pub struct Program {
    iterations: u64,
    source: Source,
    rng: DataGenRng,
    interpreter: InterpreterType,
}

impl Program {
    pub fn new<T: Into<Source>>(
        verbosity: u64,
        iterations: u64,
        source: T,
        rng: DataGenRng,
        use_v2_interpreter: bool,
    ) -> Program {
        let interpreter = if use_v2_interpreter {
            InterpreterType::V2(::v2::interpreter::Interpreter::new())
        } else {
            InterpreterType::V1(::interpreter::Interpreter::new(verbosity))
        };
        Program {
            iterations,
            source: source.into(),
            rng,
            interpreter,
        }
    }

    pub fn run(self, output: &mut DataGenOutput) -> Result<(), Error> {
        let Program {
            iterations,
            source,
            mut rng,
            interpreter,
            ..
        } = self;

        let src_string = source.read_to_str()?;
        let mut generator = match interpreter {
            InterpreterType::V1(mut int) => {
                let mut gen = int.eval_program(src_string.as_ref())?;

                for _ in 0..iterations {
                    gen.write_value(&mut rng, output)?;
                }
            }
            InterpreterType::V2(mut int) => {
                let gen = int.eval(src_string.as_ref())?;

                let mut context = ::v2::ProgramContext::new(rng);
                for _ in 0..iterations {
                    gen.write_value(&mut context, output)?;
                }
            }
        };

        output.flush().map_err(Into::into)
    }

    pub fn add_library<T: Into<Source>>(&mut self, lib_source: T) -> Result<(), Error> {
        let source = lib_source.into();
        let name = source.get_name();
        let as_str = source.read_to_str()?;

        match self.interpreter {
            InterpreterType::V1(ref mut int) => int.eval_library(as_str.as_ref()),
            InterpreterType::V2(ref mut int) => int.add_module(name, as_str.as_ref()),
        }
    }
}

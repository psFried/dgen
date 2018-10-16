use failure::Error;
use std::borrow::Cow;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// reference to a file on the local filesystem
    File(PathBuf),
    /// source is held entirely in memory
    String(String),
    /// used for the standard libraries, which are included in the binary
    Builtin(&'static str, &'static str),
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
            Source::Builtin(ref name, _) => (*name).to_owned(),
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
            Source::Builtin(_, ref builtin) => Ok((*builtin).into()),
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

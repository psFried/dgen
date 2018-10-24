use failure::Error;
use std::borrow::Cow;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use IString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// reference to a file on the local filesystem. The filename will become the module name
    File(PathBuf),
    /// source is held entirely in memory
    String(String),
    /// used for the standard libraries, which are included in the binary
    Builtin(&'static str, &'static str),
    /// source will be read dynamically from stdin. It is an error to have more than one sourceType that uses stdin
    Stdin,
}

fn default_module_name() -> IString {
    "default".into()
}

impl Source {
    pub fn get_name(&self) -> IString {
        match *self {
            Source::File(ref pb) => pb
                .file_stem()
                .map(|name| {
                    name.to_str()
                        .map(Into::into)
                        .unwrap_or_else(|| default_module_name())
                }).unwrap_or_else(|| default_module_name()),
            Source::String(_) => default_module_name(),
            Source::Builtin(ref name, _) => (*name).into(),
            Source::Stdin => default_module_name(),
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

#[cfg(test)]
mod test {
    use std::path::Path;
    use super::*;

    #[test]
    fn filename_minus_extension_is_used_as_module_name() {
        let path = Path::new("/foo/bar/baz.dgen");
        let source = Source::from(path);

        let name = source.get_name();
        assert_eq!("baz", &*name);
    }
}
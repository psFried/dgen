use failure::Error;
use std::borrow::Cow;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use IString;


#[derive(Clone, Debug, PartialEq)]
pub enum UnreadSource {
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

impl UnreadSource {
    pub fn get_name(&self) -> IString {
        match *self {
            UnreadSource::File(ref pb) => pb
                .file_stem()
                .map(|name| {
                    name.to_str()
                        .map(Into::into)
                        .unwrap_or_else(|| default_module_name())
                }).unwrap_or_else(|| default_module_name()),
            UnreadSource::String(_) => default_module_name(),
            UnreadSource::Builtin(ref name, _) => (*name).into(),
            UnreadSource::Stdin => default_module_name(),
        }
    }

    pub fn get_description(&self) -> &str {
        match *self {
            UnreadSource::File(ref pb) => pb.to_str().unwrap_or("<unknown file>"),
            UnreadSource::String(_) => "<command line input>",
            UnreadSource::Builtin(ref name, _) => name,
            UnreadSource::Stdin => "<stdin>",
        }
    }

    pub fn file<P: Into<PathBuf>>(path: P) -> UnreadSource {
        UnreadSource::File(path.into())
    }

    pub fn string<S: Into<String>>(string: S) -> UnreadSource {
        UnreadSource::String(string.into())
    }

    pub fn stdin() -> UnreadSource {
        UnreadSource::Stdin
    }

}

impl From<String> for UnreadSource {
    fn from(s: String) -> UnreadSource {
        UnreadSource::string(s)
    }
}

impl<'a> From<&'a Path> for UnreadSource {
    fn from(p: &'a Path) -> UnreadSource {
        UnreadSource::file(p)
    }
}

impl From<PathBuf> for UnreadSource {
    fn from(p: PathBuf) -> UnreadSource {
        UnreadSource::file(p)
    }
}

#[derive(Debug, PartialEq)]
pub struct Source {
    unread: UnreadSource,
    source_text: Cow<'static, str>,
}

impl Source {
    pub fn read(mut unread: UnreadSource) -> Result<Source, Error> {
        let source_text = match &mut unread {
            UnreadSource::File(ref path) => {
                use std::fs::File;
                let mut file = File::open(path)?;
                let mut buffer = String::with_capacity(512);
                file.read_to_string(&mut buffer)?;
                Cow::from(buffer)
            }
            UnreadSource::Builtin(_, src) => Cow::from(*src),
            UnreadSource::String(ref mut src) => {
                // ok, this is admittedly a little weird. We're going to swap the unread source with an empty string
                // so that we don't need to copy it. In all likelyhood any source with this type was small enough to get passed
                // on the command line, so the copy would be no big deal, but avoiding the copy will be better in the case that anyone
                // uses this as a library
                Cow::from(::std::mem::replace(src, String::new()))
            }
            UnreadSource::Stdin => {
                let mut sin = io::stdin();
                let mut buffer = String::with_capacity(32);
                sin.read_to_string(&mut buffer)?;
                Cow::from(buffer)
            }
        };

        Ok(Source {
            unread,
            source_text,
        })
    }

    pub fn text(&self) -> &str {
        &self.source_text
    }

    pub fn module_name(&self) -> IString {
        self.unread.get_name()
    }

    pub fn description(&self) -> &str {
        self.unread.get_description()
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use super::*;

    #[test]
    fn filename_minus_extension_is_used_as_module_name() {
        let path = Path::new("/foo/bar/baz.dgen");
        let source = UnreadSource::from(path);

        let name = source.get_name();
        assert_eq!("baz", &*name);
    }
}
use failure::Error;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::rc::Rc;
use ::{
    AnyFunction, Arguments, BuiltinFunctionPrototype, CreateFunctionResult, DataGenOutput,
    DynStringFun, FunctionPrototype, GenType, ProgramContext, RunnableFunction,
};
use IString;

struct RandFileReader {
    file: File,
    file_len: u64,
    delimiter: IString,
    region_offsets: Vec<u64>,
}

impl RandFileReader {
    fn create(path: IString, delimiter: IString) -> Result<RandFileReader, Error> {
        let mut file = File::open(&*path)?;
        let file_len = file.metadata()?.len();
        let region_offsets = find_region_offsets(&mut file, &*delimiter)?;
        Ok(RandFileReader {
            file,
            file_len,
            region_offsets,
            delimiter: delimiter,
        })
    }

    fn read_random_region<'a>(
        &mut self,
        rng: &mut ProgramContext,
        buffer: &'a mut Vec<u8>,
    ) -> Result<&'a str, Error> {
        let RandFileReader {
            ref mut file,
            ref file_len,
            ref region_offsets,
            ref delimiter,
        } = *self;
        let region_idx = rng.gen_range(0, region_offsets.len() + 1);
        let region_start = if region_idx == 0 {
            0
        } else {
            // the stored offset is the index of the start of the delimiter
            // so we need to add the delimiter length
            region_offsets[region_idx - 1] + delimiter.len() as u64
        };

        let nread = if region_offsets[region_idx.min(region_offsets.len() - 1)] == region_start {
            0
        } else if region_idx < (region_offsets.len() - 1) {
            // there's another region after this one, so we'll stop there
            region_offsets[region_idx] - region_start // - delimiter.len() as u64
        } else {
            *file_len - region_start // we'll just read to the end of the file
        };

        if (buffer.len() as u64) < nread {
            buffer.resize(nread as usize, 0);
        }

        let buf_slice = &mut buffer[0..nread as usize];
        file.seek(SeekFrom::Start(region_start))?;
        file.read_exact(buf_slice)?;

        ::std::str::from_utf8(buf_slice).map_err(Into::into)
    }
}

struct SelectFromFileInner {
    read_buffer: Vec<u8>,
    readers: HashMap<IString, HashMap<IString, RandFileReader>>,
}

impl SelectFromFileInner {
    fn new() -> SelectFromFileInner {
        SelectFromFileInner {
            read_buffer: Vec::with_capacity(512),
            readers: HashMap::new(),
        }
    }

    fn read(
        &mut self,
        ctx: &mut ProgramContext,
        filename: IString,
        delimiter: IString,
    ) -> Result<&str, Error> {
        let SelectFromFileInner {
            ref mut read_buffer,
            ref mut readers,
        } = *self;
        if !readers.contains_key(&filename) {
            readers.insert(filename.clone(), HashMap::new());
        }
        let readers_by_delimiter = readers.get_mut(&filename).unwrap();
        if !readers_by_delimiter.contains_key(&delimiter) {
            let reader = RandFileReader::create(filename, delimiter.clone())?;
            readers_by_delimiter.insert(delimiter.clone(), reader);
        }
        let reader = readers_by_delimiter.get_mut(&delimiter).unwrap();
        reader.read_random_region(ctx, read_buffer)
    }
}

pub struct SelectFromFile {
    file_path_gen: DynStringFun,
    delimiter_gen: DynStringFun,
    inner: RefCell<SelectFromFileInner>,
}

impl Debug for SelectFromFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SelectFromFile")
            .field("file_path", &self.file_path_gen)
            .field("delimiter", &self.delimiter_gen)
            .finish()
    }
}

impl RunnableFunction<IString> for SelectFromFile {
    fn gen_value(&self, ctx: &mut ProgramContext) -> Result<IString, Error> {
        let SelectFromFile {
            ref file_path_gen,
            ref delimiter_gen,
            ref inner,
        } = *self;

        let path = file_path_gen.gen_value(ctx)?;
        let delimiter = delimiter_gen.gen_value(ctx)?;

        let mut cell = inner.borrow_mut();
        cell.read(ctx, path, delimiter).map(Into::into)
    }

    fn write_value(
        &self,
        ctx: &mut ProgramContext,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error> {
        let SelectFromFile {
            ref file_path_gen,
            ref delimiter_gen,
            ref inner,
        } = *self;

        let path = file_path_gen.gen_value(ctx)?;
        let delimiter = delimiter_gen.gen_value(ctx)?;

        let mut cell = inner.borrow_mut();
        let str_value = cell.read(ctx, path, delimiter)?;
        output.write_string(str_value).map_err(Into::into)
    }
}

impl SelectFromFile {
    pub fn new(path: DynStringFun, delimiter: DynStringFun) -> DynStringFun {
        Rc::new(SelectFromFile {
            file_path_gen: path,
            delimiter_gen: delimiter,
            inner: RefCell::new(SelectFromFileInner::new()),
        })
    }
}

fn is_region_start(buffer: &[u8], idx: usize, delimiter: &[u8]) -> bool {
    buffer[idx..].starts_with(delimiter)
}

fn do_read<R: Read>(read: &mut R, buf: &mut [u8]) -> io::Result<usize> {
    loop {
        match read.read(buf) {
            Ok(n) => return Ok(n),
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => { /* loop around and retry */ }
            err @ _ => return err,
        }
    }
}

fn find_region_offsets(file: &mut File, delimiter: &str) -> Result<Vec<u64>, io::Error> {
    file.seek(SeekFrom::Start(0))?;
    let mut result = Vec::with_capacity(32);
    let mut buffer = [0; 8192];
    let delimiter_bytes = delimiter.as_bytes();
    let delimiter_length = delimiter_bytes.len();

    let mut carry_over_len = 0;
    let mut index_adder = 0;
    loop {
        let nread = do_read(file, &mut buffer[carry_over_len..])?;
        if nread == 0 {
            break;
        }

        let buffer_end = nread - carry_over_len;
        let mut buffer_idx = 0;
        while buffer_idx < buffer_end {
            if is_region_start(&buffer[..], buffer_idx, delimiter_bytes) {
                let resolved_idx = buffer_idx as u64 + index_adder;
                result.push(resolved_idx);
                buffer_idx += delimiter_length;
            } else {
                buffer_idx += 1;
            }
        }

        // take the last n bytes from the end of the buffer and put them back at the beginning so
        // that we can make sure to catch all the occurences of multi-byte delimiters
        for i in 0..carry_over_len {
            let end_idx = buffer_end + i;
            let byte = buffer[end_idx];
            buffer[i] = byte;
        }
        carry_over_len = delimiter_length - 1;
        index_adder += nread as u64;
    }
    Ok(result)
}


const FILEPATH_PARAM: &str = "filepath";
const DELIMITER_PARAM: &str = "delimiter";

fn create_file_fun(args: Arguments) -> CreateFunctionResult {
    let (filepath, delimiter) = args.require_2_args(
        FILEPATH_PARAM,
        AnyFunction::require_string,
        DELIMITER_PARAM,
        AnyFunction::require_string,
    )?;
    Ok(AnyFunction::String(SelectFromFile::new(
        filepath, delimiter,
    )))
}

fn create_words_fun(_: Arguments) -> CreateFunctionResult {
    use std::path::Path;
    use ::ConstString;

    let words_paths = ["/usr/share/dict/words", "/usr/dict/words"];
    let path = words_paths
        .iter()
        .filter(|path| Path::new(path).is_file())
        .next()
        .map(|path| ConstString::new(*path))
        .ok_or_else(|| {
            format_err!(
                "Could not find a words file in the usual places: {:?} Try using `select_from_file(String, String)` instead",
                words_paths
            )
        })?;
    let delimiter = ConstString::new("\n");

    let args = Arguments::new(vec![path, delimiter]);
    create_file_fun(args)
}

pub const SELECT_FROM_FILE_BUILTIN: &FunctionPrototype = &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
    function_name: "select_from_file",
    description: "Selects random regions from the given file, using the given delimiter (most commonly a newline)",
    arguments: &[
        (FILEPATH_PARAM, GenType::String),
        (DELIMITER_PARAM, GenType::String)
    ],
    variadic: false,
    create_fn: &create_file_fun,
});

pub const WORDS_BUILTIN: &FunctionPrototype = &FunctionPrototype::Builtin(&BuiltinFunctionPrototype {
    function_name: "words",
    description: "Selects a random word from the unix words file (/usr/share/dict/words or /usr/dict/words)",
    arguments: &[],
    variadic: false,
    create_fn: &create_words_fun,
});

#[cfg(test)]
mod test {
    use super::*;
    use ::ConstString;
    use verbosity;

    const RAND_SEED: &[u8; 16] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    #[test]
    fn produces_words_from_file_with_lf_line_endings() {
        let mut rng = ProgramContext::from_seed(*RAND_SEED, verbosity::NORMAL);
        let path_gen = ConstString::new("test-data/simple-words.txt")
            .require_string()
            .unwrap();
        let delim = ConstString::new("\n").require_string().unwrap();
        let subject = SelectFromFile::new(path_gen, delim);

        let expected = vec!["foo", "bar", "baz", "", "qux"];
        for _ in 0..20 {
            let actual = subject.gen_value(&mut rng).expect("failed to gen value");
            assert!(expected.contains(&&*actual));
        }
    }

    #[test]
    fn produces_words_from_file_with_crlf_line_endings() {
        let mut rng = ProgramContext::from_seed(*RAND_SEED, verbosity::NORMAL);
        let path_gen = ConstString::new("test-data/crlf-words.txt")
            .require_string()
            .unwrap();
        let delim = ConstString::new("\r\n").require_string().unwrap();
        let subject = SelectFromFile::new(path_gen, delim);

        let expected = vec!["foo", "bar", "baz", "", "qux"];
        for _ in 0..20 {
            let actual = subject.gen_value(&mut rng).expect("failed to gen value");
            assert!(expected.contains(&&*actual));
        }
    }
}

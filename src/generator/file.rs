use generator::{DataGenRng, Generator, DynStringGenerator};
use writer::DataGenOutput;
use rand::Rng;
use failure::Error;

use std::fs::File;
use std::io::{self, Seek, SeekFrom, Read};
use std::fmt::{self, Display};

pub const SELECT_FROM_FILE_FUN_NAME: &'static str = "select_from_file";

const BUFFER_SIZE: u64 = 4096;
const READ_OFFSET: u64 = 2048;

struct RandFileReader {
    file_path_generator: DynStringGenerator,
    separator_gen: DynStringGenerator,
    prev_path: String,
    file_and_len: Option<(File, u64)>,
    buffer: Vec<u8>,
}

impl RandFileReader {

    fn new(path: DynStringGenerator, delimiter: DynStringGenerator) -> RandFileReader {
        RandFileReader {
            file_path_generator: path,
            separator_gen: delimiter,
            prev_path: String::new(),
            file_and_len: None,
            buffer: vec![0; BUFFER_SIZE as usize],
        }
    }

    fn open_file(&mut self, rng: &mut DataGenRng) -> Result<(), Error> {
        let new_path = match self.file_path_generator.gen_value(rng)? {
            // TODO: impl Display for SelectLineFromFile
            None => return Err(format_err!("File path cannot be null for generator: '{}'", self.prev_path)),
            Some(p) => p
        };
        // check if the path has changed. We want to minimize the number of times we open files
        // so we'll only open it when the path changes. We always try to open if the previous path
        // is empty, since that condition can only occur on the very first invocation
        if *new_path != self.prev_path || self.prev_path.is_empty() {
            // we'll need to open a new file
            self.prev_path.clear();
            self.prev_path.push_str(new_path.as_str());

            let file = File::open(self.prev_path.as_str())?;
            let metadata = file.metadata()?;
            if !metadata.is_file() {
                return Err(format_err!("File path: '{}' is not a regular file", self.prev_path));
            }
            let file_len = metadata.len();
            self.file_and_len = Some((file, file_len));
        }
        Ok(())
    }

    fn read_value(&mut self, rng: &mut DataGenRng) -> Result<&str, Error> {
        self.open_file(rng)?;
        let RandFileReader {ref mut file_and_len, ref mut buffer, ref mut separator_gen, ref prev_path, ..} = *self;
        let (ref mut file, ref file_len) = file_and_len.as_mut().unwrap();
        let delimiter = separator_gen.gen_value(rng)?.map(|d| d.as_str()).unwrap_or("\n");

        // pick a random spot in the file to start reading
        let starting_point = rng.gen_range(0, *file_len);
        let scan_forward = rng.gen_bool(0.5);
        // back up a little bit, since we may look either forward or backward to try to find a boundary
        // we do this to avoid being biased toward either the beginning or the end of the file
        let read_start = starting_point.saturating_sub(READ_OFFSET);
        file.seek(SeekFrom::Start(read_start))?;
        let nread = do_read(file, buffer)?;

        let valid_range = &buffer[..nread];
        let start_index = (READ_OFFSET as usize).min(nread);
    
        let region = find_region(valid_range, start_index, scan_forward, delimiter);
        
        region.ok_or_else(|| {
            format_err!("Cannot find boudaries of values in file: '{}'", prev_path)
        }).and_then(|bytes| {
            ::std::str::from_utf8(bytes).map_err(Into::into)
        })
    }
}

fn find_region<'a>(buffer: &'a [u8], start_index: usize, scan_forward: bool, delimiter: &str) -> Option<&'a [u8]> {
    let delim_bytes = delimiter.as_bytes();

    let mut idx = start_index;
    while !is_region_start(buffer, idx, delim_bytes) {
        if idx == 0 || idx >= buffer.len() {
            return None;
        }
        if scan_forward {
            idx += 1;
        } else {
            idx -= 1;
        }
    }
    idx += delim_bytes.len();

    // once we've found the start region, we need to look for the end of it
    // the end of the buffer will be considered the same as the region end so that
    // we don't have to error out if, for instance, the file doesn't end with a newline
    let mut end_idx = idx + delim_bytes.len();
    while end_idx < buffer.len() && !is_region_start(buffer, end_idx, delim_bytes) {
        end_idx += 1;
    }

    Some(&buffer[idx..end_idx])
}

pub struct SelectFromFile {
    reader: RandFileReader,
    value: String,
}

impl Generator for SelectFromFile {
    type Output = String;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&String>, Error> {
        let SelectFromFile {ref mut reader, ref mut value} = *self;
        let value_str = reader.read_value(rng)?;
        value.clear();
        value.push_str(value_str);
        Ok(Some(&*value))
    }

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> Result<u64, Error> {
        let str_val: &str = self.reader.read_value(rng)?;
        output.write_string(&str_val).map_err(Into::into)
    }

    fn new_from_prototype(&self) -> DynStringGenerator {
        let path = self.reader.file_path_generator.new_from_prototype();
        let delim = self.reader.separator_gen.new_from_prototype();
        SelectFromFile::new(path, delim)
    }
}

impl SelectFromFile {
    pub fn new(path: DynStringGenerator, delimiter: DynStringGenerator) -> DynStringGenerator {
        Box::new(SelectFromFile {
            reader: RandFileReader::new(path, delimiter),
            value: String::new(),
        })
    }
}
impl Display for SelectFromFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({}, {})", SELECT_FROM_FILE_FUN_NAME, self.reader.file_path_generator, self.reader.separator_gen)
    }
}


fn is_region_start(buffer: &[u8], idx: usize, delimiter: &[u8]) -> bool {
    buffer[idx..].starts_with(delimiter)
}

fn do_read<R: Read>(read: &mut R, buf: &mut [u8]) -> io::Result<usize> {
    loop {
        match read.read(buf) {
            Ok(n) => return Ok(n),
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => { /* loop around and retry */ },
            err @ _ => return err
        }
    }
}
use generator::{DataGenRng, Generator, DynStringGenerator};
use writer::DataGenOutput;
use rand::Rng;
use failure::Error;

use std::fs::File;
use std::io::{self, Seek, SeekFrom, Read};
use std::fmt::{self, Display};
use std::collections::HashMap;

pub const SELECT_FROM_FILE_FUN_NAME: &'static str = "select_from_file";

struct RandFileReader {
    file: File,
    file_len: u64,
    delimiter: String,
    region_offsets: Vec<u64>,
}

impl RandFileReader {
    fn create(path: &str, delimiter: &str) -> Result<RandFileReader, Error> {
        let mut file = File::open(path)?;
        let file_len = file.metadata()?.len();
        let region_offsets = find_region_offsets(&mut file, delimiter)?;
        Ok(RandFileReader {
            file,
            file_len,
            region_offsets,
            delimiter: delimiter.to_owned(),
        })
    }

    fn read_random_region<'a>(&mut self, rng: &mut DataGenRng, buffer: &'a mut Vec<u8>) -> Result<&'a str, Error> {
        let RandFileReader { ref mut file, ref file_len, ref region_offsets, ref delimiter} = *self;
        let region_idx = rng.gen_range(0, region_offsets.len() - 1);
        let region_start = region_offsets[region_idx];
        let nread = if region_idx < (region_offsets.len() - 1) {
            // there's another region after this one, so we'll stop there
            region_offsets[region_idx + 1] - region_start
        } else {
            *file_len - region_start  // we'll just read to the end of the file
        };
        let nread = nread - delimiter.len() as u64;

        if (buffer.len() as u64) < nread {
            buffer.resize(nread as usize, 0);
        }

        let buf_slice = &mut buffer[0..nread as usize];
        file.seek(SeekFrom::Start(region_start))?;
        file.read_exact(buf_slice)?;

        ::std::str::from_utf8(buf_slice).map_err(Into::into)
    }
}

pub struct SelectFromFile {
    file_path_gen: DynStringGenerator,
    delimiter_gen: DynStringGenerator,
    read_buffer: Vec<u8>,
    readers: HashMap<String, HashMap<String, RandFileReader>>,
}

impl Generator for SelectFromFile {
    type Output = str;

    fn gen_value(&mut self, rng: &mut DataGenRng) -> Result<Option<&str>, Error> {
        let SelectFromFile { 
            ref mut file_path_gen, 
            ref mut delimiter_gen, 
            ref mut read_buffer,
            ref mut readers 
        } = *self;
        
        let path = file_path_gen.gen_value(rng)?.ok_or_else(|| {
            format_err!("File path cannot be null for {}", SELECT_FROM_FILE_FUN_NAME)
        })?;
        let delimiter = delimiter_gen.gen_value(rng)?.unwrap_or("\0");

        if !readers.contains_key(path) {
            readers.insert(path.to_owned(), HashMap::new());
        }
        let readers_by_delimiter = readers.get_mut(path).unwrap();
        if !readers_by_delimiter.contains_key(delimiter) {
            let reader = RandFileReader::create(path, delimiter)?;
            readers_by_delimiter.insert(delimiter.to_owned(), reader);
        }

        let reader = readers_by_delimiter.get_mut(delimiter).unwrap();
        reader.read_random_region(rng, read_buffer).map(|value| Some(value))
    }

    fn write_value(&mut self, rng: &mut DataGenRng, output: &mut DataGenOutput) -> Result<u64, Error> {
        let value = self.gen_value(rng)?;
        if let Some(s) = value {
            output.write_string(s).map_err(Into::into)
        } else {
            Ok(0)
        }
    }

    fn new_from_prototype(&self) -> DynStringGenerator {
        let path = self.file_path_gen.new_from_prototype();
        let delim = self.delimiter_gen.new_from_prototype();
        SelectFromFile::new(path, delim)
    }
}

impl SelectFromFile {
    pub fn new(path: DynStringGenerator, delimiter: DynStringGenerator) -> DynStringGenerator {
        Box::new(SelectFromFile {
            file_path_gen: path,
            delimiter_gen: delimiter,
            read_buffer: vec![0; 4096],
            readers: HashMap::new(),
        })
    }
}
impl Display for SelectFromFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({}, {})", SELECT_FROM_FILE_FUN_NAME, self.file_path_gen, self.delimiter_gen)
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

fn find_region_offsets(file: &mut File, delimiter: &str) -> Result<Vec<u64>, io::Error> {
    file.seek(SeekFrom::Start(0))?;
    let mut result = vec![0]; // always a region at the beginning
    let mut buffer = [0; 8192];
    let delimiter_bytes = delimiter.as_bytes();
    let delimiter_length = delimiter_bytes.len();

    let mut carry_over = [0; 64];
    let mut carry_over_len = 0;
    loop {
        let nread = do_read(file, &mut buffer[carry_over_len..])?;
        if nread == 0 {
            break;
        }

        if carry_over_len > 0 { // put the carry over into the beginning of the buffer
            let co = &carry_over[0..carry_over_len];
            buffer[0..carry_over_len].copy_from_slice(co);
        }

        let mut idx = 1; // start at 1 since we already put a region start at 0 
        while idx < nread {
            if is_region_start(&buffer[..], idx, delimiter_bytes) {
                result.push(idx as u64 + delimiter_length as u64);
                idx += delimiter_length;
            } else {
                idx += 1;
            }
        }
        // take the last n bytes from the current buffer and put it into the carry_over
        // we'll copy it into the beginning of the buffer on the next loop around
        let co = &buffer[(nread - delimiter_length)..nread];
        carry_over[0..delimiter_length].copy_from_slice(co);
        carry_over_len = delimiter_length;
    }
    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;
    use generator::constant::ConstantStringGenerator;
    use rand::SeedableRng;

    const RAND_SEED: &[u8; 16] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    #[test]
    fn produces_words_from_file() {
        let mut rng = DataGenRng::from_seed(*RAND_SEED);
        let path_gen = ConstantStringGenerator::new("test-data/simple-words.txt");
        let delim = ConstantStringGenerator::new("\n");
        let mut subject = SelectFromFile::new(path_gen, delim);

        let expected = vec!["foo", "bar", "baz", "", "qux"];
        for _ in 0..20 {
            let actual = subject.gen_value(&mut rng).expect("failed to gen value").unwrap();
            println!("actual='{}'", actual);
            assert!(expected.contains(&actual));
        }
    }
}
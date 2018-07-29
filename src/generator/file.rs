use failure::Error;
use generator::{DataGenRng, DynStringGenerator, Generator};
use rand::Rng;
use writer::DataGenOutput;

use std::collections::HashMap;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

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

    fn read_random_region<'a>(
        &mut self,
        rng: &mut DataGenRng,
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
            ref mut readers,
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
        reader
            .read_random_region(rng, read_buffer)
            .map(|value| Some(value))
    }

    fn write_value(
        &mut self,
        rng: &mut DataGenRng,
        output: &mut DataGenOutput,
    ) -> Result<u64, Error> {
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
        write!(
            f,
            "{}({}, {})",
            SELECT_FROM_FILE_FUN_NAME, self.file_path_gen, self.delimiter_gen
        )
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

#[cfg(test)]
mod test {
    use super::*;
    use generator::constant::ConstantStringGenerator;
    use rand::SeedableRng;

    const RAND_SEED: &[u8; 16] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    #[test]
    fn produces_words_from_file_with_lf_line_endings() {
        let mut rng = DataGenRng::from_seed(*RAND_SEED);
        let path_gen = ConstantStringGenerator::new("test-data/simple-words.txt");
        let delim = ConstantStringGenerator::new("\n");
        let mut subject = SelectFromFile::new(path_gen, delim);

        let expected = vec!["foo", "bar", "baz", "", "qux"];
        for _ in 0..20 {
            let actual = subject
                .gen_value(&mut rng)
                .expect("failed to gen value")
                .unwrap();
            assert!(expected.contains(&actual));
        }
    }

    #[test]
    fn produces_words_from_file_with_crlf_line_endings() {
        let mut rng = DataGenRng::from_seed(*RAND_SEED);
        let path_gen = ConstantStringGenerator::new("test-data/crlf-words.txt");
        let delim = ConstantStringGenerator::new("\r\n");
        let mut subject = SelectFromFile::new(path_gen, delim);

        let expected = vec!["foo", "bar", "baz", "", "qux"];
        for _ in 0..20 {
            let actual = subject
                .gen_value(&mut rng)
                .expect("failed to gen value")
                .unwrap();
            assert!(expected.contains(&actual));
        }
    }
}

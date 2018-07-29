use std::fmt::Display;
use std::io::{self, Write};

pub struct TrackingWriter<'a> {
    delegate: &'a mut Write,
    num_written: u64,
}

impl<'a> Write for TrackingWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let result = self.delegate.write(buf);
        if let Ok(num) = result.as_ref() {
            self.num_written += *num as u64;
        }
        result
    }

    fn flush(&mut self) -> io::Result<()> {
        self.delegate.flush()
    }
}

impl<'a> TrackingWriter<'a> {
    pub fn new(delegate: &'a mut Write) -> TrackingWriter<'a> {
        TrackingWriter {
            delegate,
            num_written: 0,
        }
    }

    pub fn get_num_bytes_written(&self) -> u64 {
        self.num_written
    }
}

pub struct DataGenOutput<'a> {
    writer: TrackingWriter<'a>,
}

impl<'a> DataGenOutput<'a> {
    pub fn new(writer: &'a mut Write) -> DataGenOutput<'a> {
        DataGenOutput {
            writer: TrackingWriter::new(writer),
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<u64> {
        self.writer.write_all(bytes).map(|()| bytes.len() as u64)
    }

    pub fn write_string<D: Display + ?Sized>(&mut self, value: &D) -> io::Result<u64> {
        let start = self.writer.get_num_bytes_written();
        self.writer
            .write_fmt(format_args!("{}", value))
            .map(|()| self.writer.get_num_bytes_written() - start)
    }
}

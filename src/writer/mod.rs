use encoding::ByteWriter;
use std::fmt::Display;
use std::io::{self, Write};
use crate::OutputType;
use failure::Error;

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

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.writer.write_all(bytes).map_err(|io_err| io_err.into())
    }

    pub fn write_str(&mut self, value: &str) -> Result<(), Error> {
        self.write_bytes(value.as_bytes())
    }

    pub fn write_string<D: Display + ?Sized>(&mut self, value: &D) -> Result<(), Error> {
        let _start = self.writer.get_num_bytes_written();
        self.writer
            .write_fmt(format_args!("{}", value))
            .map_err(|_| {
                format_err!("Failed to write to output")
            })
    }

    pub fn write<O: OutputType>(&mut self, value: &O) -> Result<(), Error> {
        value.write_output(self)
    }

    pub fn with<F, T>(&mut self, fun: F) -> Result<(), Error>
    where
        F: FnOnce(&mut DataGenOutput) -> Result<T, ::failure::Error>,
    {
        let _start = self.writer.get_num_bytes_written();
        let _ = fun(self)?;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<'a> ByteWriter for DataGenOutput<'a> {
    fn write_byte(&mut self, b: u8) {
        self.writer
            .write_all(&[b])
            .expect("Failed to write output of encoded string");
    }
    fn write_bytes(&mut self, v: &[u8]) {
        self.writer
            .write_all(v)
            .expect("Failed to write output of encoded string");
    }
}


impl<'a> ::std::fmt::Write for DataGenOutput<'a> {
    fn write_str(&mut self, s: &str) -> ::std::fmt::Result {
        self.write_str(s).map(|_| ()).map_err(|_| ::std::fmt::Error)
    }
}

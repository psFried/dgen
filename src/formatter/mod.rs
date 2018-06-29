mod csv;

use std::io::{self, Write};

pub trait Formatter {

    fn write_file_start(&mut self) -> io::Result<()>;
    fn write_column_header(&mut self, column_name: &str) -> io::Result<()>;


    fn write_row_start(&mut self) -> io::Result<()>;
    fn write_column_start(&mut self, column_name: &str) -> io::Result<()>;
    fn write_column_end(&mut self, column_name: &str) -> io::Result<()>;
    fn write_row_end(&mut self) -> io::Result<()>;
    fn write_file_end(&mut self) -> io::Result<()>;

    fn write_str(&mut self, value: &str) -> io::Result<()>;

    fn write_null(&mut self) -> io::Result<()>;

}


pub trait StringWriter {
    fn write_str(&mut self, value: &str) -> io::Result<()>;

    fn flush(&mut self) -> io::Result<()>;
}

// TODO: replace this basic blanket impl with something that allows for different encodings
impl <W: Write> StringWriter for W {
    fn write_str(&mut self, value: &str) -> io::Result<()> {
        self.write_all(value.as_bytes())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush()
    }
}

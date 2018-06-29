use super::{Formatter, StringWriter};

use std::io::{self, Error};


#[derive(Debug)]
pub struct CsvFormatter<W: StringWriter> {
    write_column_headers: bool,
    column_separator: String,
    line_separator: String,
    quote_character: String,
    null_representation: String,
    row_start: bool,
    sink: W,
}

impl <W: StringWriter> CsvFormatter<W> {

    fn write_column_separator(&mut self) -> io::Result<()> {
        let CsvFormatter {ref column_separator, ref mut sink, ref mut row_start, ..} = *self;
        if *row_start {
            *row_start = false;
            write_str(sink, column_separator.as_str())
        } else {
            Ok(())
        }
    }
}

fn write_str<W: StringWriter>(sink: &mut W, value: &str) -> io::Result<()> {
    sink.write_str(value)
}


impl <W: StringWriter> Formatter for CsvFormatter<W> {
    fn write_file_start(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn write_column_header(&mut self, column_name: &str) -> io::Result<()> {
        self.write_column_separator()?;
        write_str(&mut self.sink, column_name)
    }

    fn write_row_start(&mut self) -> Result<(), Error> {
        self.row_start = true;
        Ok(())
    }

    fn write_column_start(&mut self, column_name: &str) -> Result<(), Error> {
        self.write_column_separator()
    }

    fn write_column_end(&mut self, column_name: &str) -> Result<(), Error> {
        Ok(())
    }

    fn write_row_end(&mut self) -> Result<(), Error> {
        let CsvFormatter {ref mut sink, ref line_separator, ..} = *self;
        write_str(sink, line_separator.as_str())
    }

    fn write_file_end(&mut self) -> Result<(), Error> {
        self.sink.flush()
    }

    fn write_str(&mut self, value: &str) -> Result<(), Error> {
        let CsvFormatter {ref mut sink, ref quote_character, ..} = *self;
        write_str(sink, quote_character.as_str())?;
        write_str(sink, value)?;
        write_str(sink, quote_character.as_str())
    }

    fn write_null(&mut self) -> Result<(), Error> {
        let CsvFormatter {ref mut sink, ref null_representation, ..} = *self;
        write_str(sink, null_representation.as_str())
    }
}
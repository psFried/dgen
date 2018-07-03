
use std::io::{self, Write};
use std::fmt::Display;

pub trait Formatter {

    fn write_file_start(&mut self) -> io::Result<()>;
    fn write_file_end(&mut self) -> io::Result<()>;

    fn write_iteration_start(&mut self) -> io::Result<()>;
    fn write_iteration_end(&mut self) -> io::Result<()>;

    fn write_char(&mut self, value: &char) -> io::Result<()>;
    fn write_u64(&mut self, value: &u64) -> io::Result<()>;
    fn write_f64(&mut self, value: &f64) -> io::Result<()>;
    fn write_str(&mut self, value: &str) -> io::Result<()>;
    fn write_value_as_str(&mut self, value: &Display) -> io::Result<()>;

    fn write_null(&mut self) -> io::Result<()>;
}


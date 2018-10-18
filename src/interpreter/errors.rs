use ::IString;
use failure::Error;
use ::AnyFunction;
use ::FunctionPrototype;
use itertools::Itertools;
use std::fmt::{self, Display};

/// Used to display a region of a source file when there is an error
pub struct SourceErrRegion<'a> {
    source: &'a str,
    location_offset: usize,
}

impl<'a> Display for SourceErrRegion<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let line_number = self.get_line_number();

        let (region, offset_err_location) = self.get_source_err_display_region();
        // account for the space it takes to render the line number
        let offset = offset_err_location + 12; 
        write!(f, "line{:>5}| {}\n{:>offset$}\n", line_number, region, "^", offset = offset)
    }
}

impl<'a> SourceErrRegion<'a> {

    pub fn new(source: &'a str, location_offset: usize) -> SourceErrRegion {
        SourceErrRegion {
            source,
            location_offset,
        }
    }

    pub fn get_line_number(&self) -> usize {
        let err_location = self.location_offset;
        let mut line = 1;
        
        for (i, c) in self.source.char_indices() {
            if i >= err_location {
                break;
            }
            if c == '\n' {
                line += 1;
            }
        }
        line
    }

    fn get_source_err_display_region(&self) -> (&str, usize) {
        let err_location = self.location_offset;
        let source = self.source;

        let mut line_start = err_location;
        for (i, character) in source[..err_location].char_indices().rev() {
            if character == '\n' {
                break;
            }
            line_start = i;
        }

        let mut line_end = err_location;
        for character in source[err_location..].chars() {
            if character == '\n' || character == '\r' {
                break;
            }
            line_end += character.len_utf8();
        }

        let rendered = &source[line_start..line_end];
        let offset_location = err_location - line_start;
        (rendered, offset_location)
    }
}

#[test]
fn parse_err_is_displayed_correctly() {
    let source = r##"

    foo

    bar

    baz

"##;

    let location_offset = 17; // the 'r' at the end of "bar" (offset includes the indentation spaces)
    assert_eq!("r", &source[location_offset..(location_offset + 1)]);

    {
        let subject = SourceErrRegion {source, location_offset};
        let rendered = format!("{}", subject);
        /*
        Should render with the caret underneath the problem character in the terminal as:
line    5: bar
             ^
        */
        let expected = "line    5|     bar\n                 ^\n";
        assert_eq!(expected, rendered.as_str());
    }

    let location_offset = 6; // the 'f' in "foo"
    let subject = SourceErrRegion {source, location_offset};
    let rendered = format!("{}", subject);
    /*
    Should render with the caret underneath the problem character in the terminal as:
line    3: foo
           ^
    */
    let expected = "line    3|     foo\n               ^\n";
    assert_eq!(expected, rendered.as_str());
}

pub fn no_such_argument(name: IString) -> Error {
    format_err!("No such argument: '{}'", name)
}

pub fn no_such_method(name: IString, arguments: &[AnyFunction]) -> Error {
    use itertools::Itertools;
    format_err!("No such method: '{}({})'", name, arguments.iter().map(|a| a.get_type()).join(", "))
}

pub fn ambiguous_varargs_functions(name: IString, arguments: &[AnyFunction], option1: &FunctionPrototype, option2: &FunctionPrototype) -> Error {
    let actual_arg_types = arguments.iter().map(AnyFunction::get_type).join(", ");

    format_err!("Ambiguous function call: '{}({})' could refer to two or more function prototypes:\nA: {}\nB: {}\n",
        name, actual_arg_types, option1, option2)
}
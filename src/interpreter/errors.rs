use crate::IString;
use failure::Error;
use crate::AnyFunction;
use crate::FunctionPrototype;
use itertools::Itertools;
use std::fmt::{self, Display};
use std::sync::Arc;
use crate::interpreter::ast::{GenType, Span};
use crate::interpreter::Source;


#[derive(Debug, Clone, PartialEq)]
pub struct SourceRef {
    pub source: Arc<Source>,
    pub span: Span,
}

impl SourceRef {
    pub fn new(source: Arc<Source>, span: Span) -> SourceRef {
        SourceRef { source, span }
    }

    pub fn start_line_number(&self) -> usize {
        let start_offset = self.span.start;
        let source_str = self.source.text();
        let err_region = SourceErrRegion::new(source_str, start_offset);
        err_region.get_line_number()
    }

    pub fn description(&self) -> &str {
        self.source.description()
    }

    pub fn module_name(&self) -> IString {
        self.source.module_name()
    }
}

impl Display for SourceRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let source_name = self.source.description();
        let source_str = self.source.text();

        if f.alternate() {
            let region = &source_str[self.span.start..self.span.end];
            write!(f, "{}", region)
        } else {
            let region = SourceErrRegion::new(&source_str, self.span.start);
            let line_number = region.get_line_number();

            write!(f, "{}:{}\n\n{}\n", source_name, line_number, region)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShortSourceDescription {
    Builtin,
    Interpreted(SourceRef)
}

impl From<Option<SourceRef>> for ShortSourceDescription {
    fn from(maybe_source: Option<SourceRef>) -> ShortSourceDescription {
        maybe_source.map(|src| ShortSourceDescription::Interpreted(src)).unwrap_or(ShortSourceDescription::Builtin)
    }
}

impl Display for ShortSourceDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ShortSourceDescription::Builtin => f.write_str("<builtin function>"),
            ShortSourceDescription::Interpreted(ref source) => write!(f, "at {}:{}", source.description(), source.start_line_number())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorFunctionSignature {
    pub function_name: IString,
    pub arg_types: Vec<GenType>,
    pub render_variadic: bool,
    pub source: Option<ShortSourceDescription>,
}

impl ErrorFunctionSignature {
    fn from_actual_args(function_name: IString, args: &[AnyFunction]) -> ErrorFunctionSignature {
        let arg_types = args.iter().map(AnyFunction::get_type).collect();
        ErrorFunctionSignature {
            function_name,
            arg_types,
            render_variadic: false,
            source: None
        }
    }
}

impl<'a> From<&'a FunctionPrototype> for ErrorFunctionSignature {
    fn from(prototype: &'a FunctionPrototype) -> ErrorFunctionSignature {
        let function_name = prototype.name().into();
        let arg_types = prototype.collect_argument_types();
        ErrorFunctionSignature {
            function_name,
            arg_types,
            render_variadic: prototype.is_variadic(),
            source: Some(ShortSourceDescription::from(prototype.get_source())),
        }
    }
}

impl Display for ErrorFunctionSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let arg_types = self.arg_types.iter().join(", ");
        let maybe_variadic = if self.render_variadic {
            "..."
        } else {
            ""
        };
        if let Some(source) = self.source.as_ref() {
            write!(f, "{}({}{}) - {}", self.function_name, arg_types, maybe_variadic, source)
        } else {
            write!(f, "{}({}{})", self.function_name, arg_types, maybe_variadic)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AmbiguousCall {
    pub called: ErrorFunctionSignature,
    pub option1: ErrorFunctionSignature,
    pub option2: ErrorFunctionSignature,
}

impl AmbiguousCall {
    fn new(function_name: IString, actual_args: &[AnyFunction], option1: &FunctionPrototype, option2: &FunctionPrototype) -> AmbiguousCall {
        AmbiguousCall {
            called: ErrorFunctionSignature::from_actual_args(function_name, actual_args),
            option1: option1.into(),
            option2: option2.into(),
        }
    }
}

impl Display for AmbiguousCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Called function: {}\nOption A: {}\nOption B: {}", 
                self.called,
                self.option1,
                self.option2)
    }
}

#[derive(Debug)]
pub enum ErrorType {
    NoSuchArgument(IString),
    NoSuchMethod(ErrorFunctionSignature),
    NoSuchModule(IString),
    AmbiguousFunctionCall(AmbiguousCall),
    InternalError(Error),
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorType::NoSuchArgument(ref name) => {
                write!(f, "No such argument '{}' is in scope here", name)
            }
            ErrorType::NoSuchMethod(ref signature) => {
                write!(f, "No such method with signature: {}", signature)
            }
            ErrorType::NoSuchModule(ref name) => {
                write!(f, "No such module: '{}'", name)
            }
            ErrorType::AmbiguousFunctionCall(ref call) => {
                write!(f, "Ambiguous function call, which could refer to multiple functions:\n{}", call)
            }
            ErrorType::InternalError(ref err) => {
                write!(f, "Internal Error: {}", err)
            }
        }
    }
}

#[derive(Debug, Fail)]
pub struct CompileError {
    source: SourceRef,
    error_type: ErrorType,
}

impl CompileError {

    fn new(source: SourceRef, error_type: ErrorType) -> CompileError {
        CompileError {
            source,
            error_type,
        }
    }

    pub fn internal_error(err: Error, source: SourceRef) -> CompileError {
        CompileError::new(source, ErrorType::InternalError(err))
    }

    pub fn no_such_argument(name: IString, source_ref: SourceRef) -> CompileError {
        CompileError::new(source_ref, ErrorType::NoSuchArgument(name))
    }

    pub fn no_such_module(name: IString, source_ref: SourceRef) -> CompileError {
        CompileError::new(source_ref, ErrorType::NoSuchModule(name))
    }

    pub fn no_such_method(name: IString, arguments: &[AnyFunction], source_ref: SourceRef) -> CompileError {
        let error_type = ErrorType::NoSuchMethod(ErrorFunctionSignature::from_actual_args(name, arguments));
        CompileError::new(source_ref, error_type)
    }

    pub fn ambiguous_function_call(name: IString, arguments: &[AnyFunction], option1: &FunctionPrototype, option2: &FunctionPrototype, source_ref: SourceRef) -> CompileError {
        let call = AmbiguousCall::new(name, arguments, option1, option2);
        let error_type = ErrorType::AmbiguousFunctionCall(call);
        CompileError::new(source_ref, error_type)
    }

    pub fn get_type(&self) -> &ErrorType {
        &self.error_type
    }
}

impl Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Compilation Error: {}\n\n{}\n", self.error_type, self.source)
    }
}


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

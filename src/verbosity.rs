

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub struct Verbosity(u32);

impl From<u32> for Verbosity {
    fn from(verbosity: u32) -> Verbosity {
        Verbosity(verbosity)
    }
}

/// Nothing will ever get printed to stderr
pub const SILENT: Verbosity = Verbosity(0);

/// Only brief error messages will be printed to stderr
pub const QUIET: Verbosity = Verbosity(1);

/// Normal error messages and stacktraces will be printed to stderr
pub const NORMAL: Verbosity = Verbosity(2);

/// Print additional debug info
pub const VERBOSE: Verbosity = Verbosity(3);

/// Print dgen internal debug info and stacktraces
pub const DGEN_DEBUG: Verbosity = Verbosity(4);

impl Verbosity {

    pub fn should_print_error(&self) -> bool {
        *self >= QUIET
    }

    pub fn should_print_stacktrace(&self) -> bool {
        *self >= NORMAL
    }

    pub fn is_verbose(&self) -> bool {
        *self >= VERBOSE
    }

    pub fn should_print_debug_stacktrace(&self) -> bool {
        *self >= DGEN_DEBUG
    }

}
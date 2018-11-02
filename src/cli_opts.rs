use std::path::PathBuf;
use dgen::verbosity::Verbosity;

#[derive(Debug, StructOpt)]
#[structopt(name = "dgen", about = "A language and interpreter for generating pseudo-random data", after_help = "Run `dgen <subcommand> --help` for help on specific subcommands\nrun `dgen` without any arguments to enter the interactive shell")]
pub struct CliOptions {
    /// Output move information to stderr. Multiple occurrences will increase the verbosity. Contradicts `quiet` if both are supplied.
    /// The default verbosity will print errors and stacktraces to stderr.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences), raw(global = "true"))]
    pub verbose: u32,

    /// Make logging to stderr quieter. Contradicts `verbose` if both are supplied, so `dgen -VVV -qqq` would result in normal output.
    /// Normally, two `-q`s is enough to suppress all stderr output.
    #[structopt(short = "q", long = "quiet", parse(from_occurrences), raw(global = "true"))]
    pub quiet: u32,

    #[structopt(subcommand)]
    pub subcommand: Option<SubCommand>,

    /// Shortcut for `dgen run --lib file1 --lib file2 -f fileN`
    #[structopt(parse(from_os_str))]
    pub files: Vec<PathBuf>,
}

impl CliOptions {
    pub fn get_verbosity(&self) -> Verbosity {
        let start = self.verbose + 2; // 2 is the default verbosity. `verbose` and `quiet` will adjust from there
        start.saturating_sub(self.quiet).into()
    }
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    /// Print information on available functions
    #[structopt(name = "help")]
    Help {
        /// Print information on all functions whose name contains the given string
        #[structopt(short = "f", long = "function")]
        function_name: Option<String>,

        /// Print information on a specific module
        #[structopt(short = "m", long = "module")]
        module_name: Option<String>,
    },

    /// Run a program to generate some data
    #[structopt(name = "run")]
    RunProgram {
        /// Specification of how to generate data.{n}
        /// The language syntax is intentionally very minimal and simple. There are only two types of expressions,
        /// literals and function calls. {n}
        ///
        /// Literals: {n}
        /// - String: Anything surrounded by double quotes, for example `"foo"`. Quote characters that appear inside
        ///           the string must be escaped with a backslash character (\).{n}
        /// - Unsigned Integer: Any integer literal, for example `123` or `99`. {n}
        /// - Decimal: A decimal number, for example `3.14` {n}
        /// {n}
        /// Function Calls: are made using the form functionName(arg1, arg2, argn) {n}
        /// Arguments are positional and strongly typed (though no type annotations are used in the language syntax).) {n}
        /// Variadic functions are supported.
        ///
        #[structopt(short = "p", long = "program")]
        program: Option<String>,

        /// Read the generator program from the given file
        #[structopt(short = "f", long = "program-file", parse(from_os_str))]
        program_file: Option<PathBuf>,

        // read the generator program from stdin
        #[structopt(long = "stdin")]
        stdin: bool,

        /// number of iterations to print
        #[structopt(short = "n", long = "iterations", default_value = "1")]
        iteration_count: u64,

        /// Add the given library file to the program scope. Libraries are evaluated in the order given, and all libraries will
        /// be evaluated prior to evaluating the program. The standard library is always evaluated and in scope.
        #[structopt(short = "l", long = "lib", parse(from_os_str))]
        libraries: Vec<PathBuf>,

        /// Do not use the standard library. Useful if you don't use standard library functions and want to keep them out of the
        /// global scope. Note that builtin functions will always be in the global scope.
        #[structopt(long = "no-std")]
        no_std_lib: bool,

        /// specifies the seed used for the random number generator. For an identical program, the same seed will always produce
        /// exactly the same results. This is NOT guaranteed to be true for different versions of dgen, though.
        #[structopt(short = "s", long = "seed")]
        seed: Option<String>,
    },
}

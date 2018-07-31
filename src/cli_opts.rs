use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "datagen", about = "Generate random data sets")]
pub struct CliOptions {
    /// Enable debug logging to stderr. Multiple occurrences will increase the verbosity
    #[structopt(short = "V", parse(from_occurrences))]
    pub debug: u64,

    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    /// Print information on available functions
    #[structopt(name = "help")]
    ListFunctions {
        #[structopt(short = "f", long = "function")]
        name: Option<String>,
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
        /// exactly the same results. This is NOT guaranteed to be true for different versions of pgen, though.
        #[structopt(short = "s", long = "seed")]
        seed: Option<String>,
    },
}

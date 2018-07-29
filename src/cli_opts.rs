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
        #[structopt(short = "s", long = "stdin")]
        stdin: bool,

        /// number of iterations to print
        #[structopt(short = "n", long = "iterations", default_value = "1")]
        iteration_count: u64,
    },
}

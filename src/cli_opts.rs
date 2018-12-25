use dgen::interpreter::UnreadSource;
use dgen::verbosity::Verbosity;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "dgen",
    about = "A language and interpreter for generating pseudo-random data",
    after_help = "Run `dgen` without any arguments to enter the interactive shell\nRun `dgen help` to get information on available modules and functions"
)]
pub struct CliOptions {
    /// Output move information to stderr. Multiple occurrences will increase the verbosity. Contradicts `quiet` if both are supplied.
    /// The default verbosity will print errors and stacktraces to stderr.
    #[structopt(
        short = "v",
        long = "verbose",
        parse(from_occurrences),
        raw(global = "true")
    )]
    pub verbose: u32,

    /// Make logging to stderr quieter. Contradicts `verbose` if both are supplied, so `dgen -VVV -qqq` would result in normal output.
    /// Normally, two `-q`s is enough to suppress all stderr output.
    #[structopt(
        short = "q",
        long = "quiet",
        parse(from_occurrences),
        raw(global = "true")
    )]
    pub quiet: u32,

    /// Shortcut for `dgen --lib file1 --lib file2 -f fileN`
    #[structopt(parse(from_os_str))]
    pub files: Vec<PathBuf>,

    /// Run the program provided as an argument. Useful for short, simple programs, for example `-p 'repeat(5, words())'`
    ///
    #[structopt(short = "p", long = "program")]
    pub program: Option<String>,

    /// Read the program from the given file. This file is expected to end with a single expression, which will be run
    #[structopt(
        short = "f",
        long = "program-file",
        parse(from_os_str),
        raw(global = "true")
    )]
    pub program_file: Option<PathBuf>,

    // read the program from stdin
    #[structopt(long = "stdin")]
    pub stdin: bool,

    /// Run the program n times (default of 1)
    #[structopt(short = "n", long = "iterations", default_value = "1")]
    pub iteration_count: u64,

    /// Add the given library file to the program scope. Libraries are evaluated in the order given, and all libraries will
    /// be evaluated prior to evaluating the program. The standard library is always evaluated and in scope.
    #[structopt(
        short = "l",
        long = "lib",
        parse(from_os_str),
        raw(global = "true")
    )]
    pub libraries: Vec<PathBuf>,

    /// Do not use the standard library. Useful if you don't use standard library functions and want to keep them out of the
    /// global scope. Note that builtin functions will always be in the global scope.
    #[structopt(long = "no-std")]
    pub no_std_lib: bool,

    /// specifies the seed used for the random number generator. For an identical program, the same seed will always produce
    /// exactly the same results. This is NOT guaranteed to be true for different versions of dgen, though.
    #[structopt(short = "s", long = "seed")]
    pub seed: Option<String>,

    #[structopt(subcommand)]
    pub subcommand: Option<SubCommand>,
}

impl CliOptions {
    pub fn get_verbosity(&self) -> Verbosity {
        let start = self.verbose + 2; // 2 is the default verbosity. `verbose` and `quiet` will adjust from there
        start.saturating_sub(self.quiet).into()
    }

    pub fn get_program_source(&self) -> Option<UnreadSource> {
        if self.stdin {
            Some(UnreadSource::stdin())
        } else if !self.files.is_empty() {
            Some(UnreadSource::file(
                self.files.iter().last().cloned().unwrap(),
            ))
        } else if self.program.is_some() {
            self.program
                .as_ref()
                .cloned()
                .map(|s| UnreadSource::string(s))
        } else {
            self.program_file
                .as_ref()
                .cloned()
                .map(|f| UnreadSource::file(f))
        }
    }

    pub fn get_library_sources(&self) -> Vec<UnreadSource> {
        let mut libs = Vec::with_capacity(self.libraries.len().max(self.files.len()));

        for lib in self.libraries.iter() {
            let src = UnreadSource::file(lib.clone());
            libs.push(src);
        }

        let num_files = self.files.len();
        if num_files > 1 {
            // add all files except the last
            for file in self.files[0..(num_files - 1)].iter() {
                let src = UnreadSource::file(file.clone());
                libs.push(src);
            }
        }
        libs
    }
}

#[derive(Debug, StructOpt)]
pub struct HelpOptions {
    /// Print information on all functions whose name contains the given string
    #[structopt(short = "F", long = "function")]
    pub function_name: Option<String>,

    /// Print information on a specific module
    #[structopt(short = "m", long = "module")]
    pub module_name: Option<String>,
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    /// Print information on available functions
    #[structopt(name = "help")]
    Help(HelpOptions),
}

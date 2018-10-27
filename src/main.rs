#[macro_use]
extern crate structopt;
#[macro_use]
extern crate failure;
extern crate itertools;
extern crate lalrpop_util;
extern crate rand;
extern crate regex;
extern crate string_cache;
extern crate dgen;

mod cli_opts;

use self::cli_opts::{CliOptions, SubCommand};
use dgen::program::Runner;
use dgen::interpreter::{Interpreter, UnreadSource, Module};
use dgen::{ProgramContext, FunctionPrototype};
use failure::Error;
use std::path::PathBuf;
use structopt::StructOpt;
use dgen::verbosity::Verbosity;


trait OrBail<T> {
    fn or_bail(self, verbosity: Verbosity) -> T;
}

impl<T> OrBail<T> for Result<T, Error> {
    fn or_bail(self, verbosity: Verbosity) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                if verbosity.should_print_error() {
                    eprintln!("Error: {}", e);
                }
                if verbosity.should_print_debug_stacktrace() {
                    eprintln!("cause: {}", e.cause());
                    eprintln!("backtrace: {}", e.backtrace());
                }
                ::std::process::exit(1);
            }
        }
    }
}

fn main() {
    // this call will print help and exit if --help is passed or args are invalid
    let args = CliOptions::from_args();
    let verbosity = args.get_verbosity();
    if verbosity.should_print_debug_stacktrace() {
        // backtraces won't get generated unless this variable is set
        std::env::set_var("RUST_BACKTRACE", "1")
    }
    //eprintln!("sli args: {:#?}", args);
    let CliOptions {subcommand, mut files, ..} = args;
    match subcommand {
        Some(SubCommand::Help { function_name, module_name }) => print_function_help(function_name, module_name, verbosity),
        Some(SubCommand::RunProgram {
            program,
            iteration_count,
            program_file,
            stdin,
            libraries,
            no_std_lib,
            seed,
        }) => {
            let source = get_program_source(program, program_file, stdin).or_bail(verbosity);
            let rng = create_rng(seed, verbosity);
            let program = create_program(
                source,
                verbosity,
                iteration_count,
                libraries,
                rng,
                !no_std_lib,
            ).or_bail(verbosity);
            run_program(program).or_bail(verbosity)
        }
        None => {
            let program_file = files.pop().ok_or_else(|| {
                format_err!("Must supply either a subcommand or a file to execute")
            }).or_bail(verbosity);

            let source = UnreadSource::file(program_file);
            let runtime_context = ProgramContext::from_random_seed(verbosity);
            let program = create_program(
                source,
                verbosity,
                1,
                files,
                runtime_context,
                true // to add the std library. This default can only be disabled when using the `run` subcommand
            ).or_bail(verbosity);
            run_program(program).or_bail(verbosity)
        }
    }
}

fn create_rng(seed: Option<String>, verbosity: Verbosity) -> ProgramContext {
    seed.map(|s| {
        let resolved_seed = string_to_byte_array(s);
        ProgramContext::from_seed(resolved_seed, verbosity)
    }).unwrap_or_else(|| ProgramContext::from_random_seed(verbosity))
}

fn string_to_byte_array(string: String) -> [u8; 16] {
    let mut result = [0u8; 16];
    for (i, byte) in string.as_bytes().iter().enumerate().take(16) {
        result[i] = *byte;
    }
    result
}

fn get_program_source(
    program_string: Option<String>,
    program_file: Option<PathBuf>,
    stdin: bool,
) -> Result<UnreadSource, Error> {
    let maybe_source = if stdin {
        Some(UnreadSource::stdin())
    } else if program_string.is_some() {
        program_string.map(Into::into)
    } else if program_file.is_some() {
        program_file.map(Into::into)
    } else {
        None
    };

    maybe_source.ok_or_else(|| format_err!("Must specify one of program, program-file, or stdin"))
}

fn create_program(
    program_source: UnreadSource,
    verbosity: Verbosity,
    iterations: u64,
    libraries: Vec<PathBuf>,
    rng: ProgramContext,
    add_std_lib: bool,
) -> Result<Runner, Error> {
    let mut program = Runner::new(verbosity, iterations, program_source, rng);

    if add_std_lib {
        program.add_std_lib();
    }
    for lib in libraries {
        program.add_library(UnreadSource::file(lib))?;
    }
    Ok(program)
}

fn has_matching_module(interpreter: &Interpreter, module_name: &str) -> Result<(), ()> {
    if interpreter.module_iterator().any(|m| {
        m.name.contains(module_name)
    }) {
        Ok(())
    } else {
        Err(())
    }
}

fn list_modules(interpreter: &Interpreter) -> String {
    use itertools::Itertools;
    interpreter.module_iterator().map(|m| m.name.clone()).join("\n")
}

fn find_modules<'a>(interpreter: &'a Interpreter, module_name: &'a str, verbosity: Verbosity) -> impl Iterator<Item = &'a Module> {
    let _ = has_matching_module(interpreter, module_name).map_err(|_| {
        let other_modules = list_modules(interpreter);
        format_err!("No module exists with name matching '{}'. Available modules are: \n\n{}\n", module_name, other_modules)
    }).or_bail(verbosity);

    interpreter.module_iterator().filter(move |m| {
        m.name.contains(module_name)
    })
}

fn print_function_help(function_name: Option<String>, module_name: Option<String>, verbosity: Verbosity) {
    let mut interpreter = Interpreter::new();
    interpreter.add_std_lib();

    match (module_name, function_name) {
        (Some(module), Some(function)) => {
            let iter = find_modules(&interpreter, module.as_str(), verbosity);
            for actual_module in iter {
                println!("\nModule: {}", actual_module.name);
                list_functions(actual_module.function_iterator(), Some(function.as_str()), verbosity);
            }
        }
        (Some(module), None) => {
            let iter = find_modules(&interpreter, module.as_str(), verbosity);
            for actual_module in iter {
                println!("\nModule: {}", actual_module.name);
                list_functions(actual_module.function_iterator(), None, verbosity);
            }
        }
        (None, Some(function)) => {
            for module in interpreter.module_iterator() {
                println!("\nModule: {}", module.name);
                list_functions(module.function_iterator(), Some(function.as_str()), verbosity);
            }
        }
        _ => {
            // print some generic help and a listing of modules
            println!("Available dgen modules: \n");
            for module in interpreter.module_iterator() {
                println!("{}", module.name);
            }
            println!("\nTo list all the functions in a specific module, run `dgen help --module <name>`");
        }
    }
}

fn list_functions<'a, 'b, I: Iterator<Item=&'a FunctionPrototype>>(function_iterator: I, function_name: Option<&'b str>, verbosity: Verbosity) {
    use std::io::{stdout, Write};

    let out = stdout();
    let mut lock = out.lock();

    let mut filtered = function_iterator.filter(|fun| {
        function_name.as_ref().map(|name| {
            fun.name().contains(*name)
        }).unwrap_or(true)
    }).peekable();

    if filtered.peek().is_none() {
        writeln!(&mut lock, "No matching functions").map_err(Into::into).or_bail(verbosity);
    } else {
        writeln!(&mut lock, "").map_err(Into::into).or_bail(verbosity);
        for fun in filtered {
            if verbosity.is_verbose() {
                writeln!(&mut lock, "{:#}", fun).map_err(Into::into).or_bail(verbosity);
            } else {
                writeln!(&mut lock, "{}", fun).map_err(Into::into).or_bail(verbosity);
            }
        }
    }
}

fn run_program(program: Runner) -> Result<(), Error> {
    let sout = std::io::stdout();
    // lock stdout once at the beginning so we don't have to keep locking/unlocking it
    let mut lock = sout.lock();
    let mut output = dgen::DataGenOutput::new(&mut lock);

    program.run(&mut output)
}

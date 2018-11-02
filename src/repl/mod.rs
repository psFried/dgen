use failure::Error;
use interpreter::{DgenParseError, Interpreter, UnreadSource};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::io::{self, Write};
use {AnyFunction, DataGenOutput, ProgramContext};

const MAX_EMPTY_LINES: u32 = 2;

pub struct Repl {
    context: ProgramContext,
    interpreter: Interpreter,
    editor: Editor<()>,
    module_source: String,
    partial_source: String,
    consecutive_blank_lines: u32,
    awaiting_incomplete_input: bool,
}

const MODULE_NAME: &str = "default";

fn execute_fn(function: AnyFunction, context: &mut ProgramContext) -> Result<(), Error> {
    let out = io::stdout();
    let mut lock = out.lock();

    let result = {
        let mut dgen_out = DataGenOutput::new(&mut lock);
        function.write_value(context, &mut dgen_out).and_then(|_| {
            // need to write a newline at the end to ensure that the last line of output doesn't get clobbered
            // by the next readline prompt
            dgen_out
                .write_str("\n")
                .and_then(|_| {
                    // probably not necessary but it's good to do the write thing
                    dgen_out.flush().map_err(Into::into)
                }).map_err(Into::into)
        })
    };

    if let Err(err) = result {
        writeln!(lock, "Program Error: {}", err)?;
    }
    Ok(())
}

impl Repl {
    pub fn new(context: ProgramContext) -> Repl {
        let mut interpreter = Interpreter::new();
        interpreter.add_std_lib();

        Repl {
            context,
            interpreter,
            editor: Editor::new(),
            module_source: String::with_capacity(1024),
            partial_source: String::with_capacity(512),
            consecutive_blank_lines: 0,
            awaiting_incomplete_input: false,
        }
    }

    pub fn run(mut self) -> Result<(), Error> {
        loop {
            let prompt = if self.awaiting_incomplete_input {
                "> ... "
            } else {
                "> "
            };
            match self.editor.readline(prompt) {
                Ok(next_line) => {
                    self.handle_new_input(next_line)?;
                }
                Err(ReadlineError::Eof) => return Ok(()),
                Err(ReadlineError::Interrupted) => return Ok(()),
                Err(ReadlineError::Utf8Error) => bail!("UTF8 Error"),
                Err(ReadlineError::Io(e)) => return Err(e.into()),
                Err(ReadlineError::Errno(e)) => bail!("Syscall Error: {}", e),
            };
        }
    }

    fn handle_new_input(&mut self, new_input: String) -> Result<(), Error> {
        if self.handle_meta_command(new_input.as_str()) {
            println!("Handled metacommand");
            return Ok(());
        }

        self.partial_source.push_str(new_input.as_str());

        if new_input.as_str().trim().is_empty() && self.consecutive_blank_lines < MAX_EMPTY_LINES {
            self.consecutive_blank_lines += 1;
            return Ok(());
        } else {
            self.awaiting_incomplete_input = false;
        }

        let mut new_combined_input = self.module_source.clone();
        new_combined_input.push_str("\n");
        new_combined_input.push_str(self.partial_source.as_str());

        self.interpreter.remove_module(MODULE_NAME);
        let result = self
            .interpreter
            .eval_any(UnreadSource::String(new_combined_input));
        match result {
            Ok(Some(function)) => {
                let mut command = String::with_capacity(32);
                ::std::mem::swap(&mut command, &mut self.partial_source);
                self.editor.add_history_entry(command);
                execute_fn(function, &mut self.context)?;
            }
            Ok(None) => {
                println!("Added function");
                self.push_partial_source_to_module();
            }
            Err(ref err) if is_unexpected_eof_parse_err(err) => {
                if self.consecutive_blank_lines >= MAX_EMPTY_LINES {
                    self.partial_source.clear();
                    println!("Error: {}", err);
                } else {
                    self.awaiting_incomplete_input = true;
                }
            }
            Err(err) => {
                self.partial_source.clear();
                println!("Error: {}", err);
            }
        }
        self.consecutive_blank_lines = 0;
        Ok(())
    }

    fn clear(&mut self) {
        self.interpreter.remove_module(MODULE_NAME);
        self.module_source.clear();
        self.partial_source.clear();
    }

    fn show(&mut self) {
        println!("# Current Source: \n {}", self.module_source);

        if let Some(m) = self.interpreter.get_module(MODULE_NAME) {
            println!("# Functions in scope: ");
            for fun in m.function_iterator() {
                println!("{}", fun);
            }
        } else {
            println!("# No functions in module");
        }
    }

    fn help(&mut self) {
        println!("{}", HELP_TXT);
    }

    fn handle_meta_command(&mut self, line: &str) -> bool {
        match get_metacommand(line) {
            Some(MetaCommand::Clear) => {
                self.clear();
                true
            }
            Some(MetaCommand::Show) => {
                self.show();
                true
            }
            Some(MetaCommand::Help) => {
                self.help();
                true
            }
            None => false,
        }
    }

    fn push_partial_source_to_module(&mut self) {
        self.module_source.push_str("\n");
        self.module_source.push_str(self.partial_source.as_str());
        self.partial_source.clear();
    }
}

fn get_second_word(line: &str) -> Option<&str> {
    let line = line.trim();
    line.split("\\s+").nth(1)
}

#[derive(Debug)]
enum MetaCommand {
    Clear,
    Show,
    Help,
}

impl ::std::str::FromStr for MetaCommand {
    type Err = ();
    fn from_str(val: &str) -> Result<MetaCommand, ()> {
        match val {
            "clear" => Ok(MetaCommand::Clear),
            "show" => Ok(MetaCommand::Show),
            "help" => Ok(MetaCommand::Help),
            _ => Err(()),
        }
    }
}

fn get_metacommand(line: &str) -> Option<MetaCommand> {
    let line = line.trim();
    line.split("\\s+")
        .nth(0)
        .and_then(|split| split.parse().ok())
}

fn is_unexpected_eof_parse_err(err: &Error) -> bool {
    if let Some(parse_err) = err.downcast_ref::<DgenParseError>() {
        if parse_err.is_unexpected_eof() {
            return true;
        }
    }
    false
}

const HELP_TXT: &str = r##"dgen shell:
You can type dgen code as you normally would. All language features are supported.
Special commands are:

show -> prints the current module source
clear -> clears all  functions that have been declared
help -> prints this message
"##;

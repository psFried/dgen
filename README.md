# dgen

[![Build Status](https://travis-ci.com/psFried/dgen.svg?branch=master)](https://travis-ci.com/psFried/dgen)

Generate evil test data

dgen is a CLI tool for generating pseudorandom data in arbitrary formats. The goal is to have a tool that works equally well for both textual and binary formats. Dgen is really just an interpreter for a simple domain-specific functional language. The syntax is documented [here](SYNTAX.md).
The language documentation is pretty sparse at the moment, but this will hopefully be improved shortly. PRs are welcome!

# Example

The following program will print 10 random quoted words, separated by newlines:

```
$ dgen run -p 'repeat_delimited(10, double_quote(words()), "\n")'
"gule"
"erugation"
"avouchment"
"hymnless"
"reclusory"
"deferral"
"debauched"
"unenervated"
"charkhana"
"sheatfish"
```

dgen programs are invoked by calling `dgen run` and providing the program input in one of several ways:

- `-p`, `--program`: the program can be provided on the command line. This is easiest for simple expressions.
- `-f`, `--program-file`: interpret the given file as a program. Nice for more complex programs
- `-s`, `--stdin`: read the program from stdin

You can also add your own libraries to the program scope using the `--lib` option.

`dgen file1 file2 fileN` can also be used as a shortcut for `dgen run --lib file1 --lib file2 -f fileN`. This allows you to run an executable dgen script by simply putting a shebang (`#!dgen`) at the top of the file.

dGen has a bunch of builtin functions, too. You can list the builtin functions by executing `dgen help`. You can optionally filter the list of functions by name with `dgen help --function <name>`. Of course `dgen -h` will print out info on all of the available options.

Take a look at [the examples](dgen_examples/) for more.

## Goals

- Make it easy to generate files and streams of data for testing
- Make it easy to share and re-use programs for data generation
- Make it easy to test data that can be represented in multiple ways
- Easily integrate with various testing tools and workflows

## Non-Goals

- Be a general purpose language. Currently the language is not even turing-complete, and it's not really clear that there would be any benefit to turing-completeness.
- Understand the samantics of your data. DGen is more focused on how data is _represented_ rather than what it _means_.

# What's differnt about dgen?

DGen focuses on how data is _represented_. Take JSON for example. It's easy to find random data generators that will output the data as well formed JSON. But if you're testing a JSON parser, then you want to use JSON with different and inconsistent formatting! Like many other formats, JSON has many _valid_ ways to represent the same data. You might want to test keys that are sometimes surrounded with double-quotes and sometimes single-quotes or unquoted. DGen is meant to fill this gap in between a well-formed dataset generator (which will typically always produce consistently formatted representations) and a fuzz tester (which is more useful for testing _invalid_ input).

### Build

dgen is built with [Rust](https://www.rust-lang.org/), and requires version 1.26 or later. Just a simple `cargo build --release` is all it takes to get a release binary. The build is currently tested only on OSX and GNU/Linux, but the intent is to start testing on Windows as well soon™️.

### Stability

This project is still in the super early stages, so major breaking changes can happen at any time. I'm still experimenting with various aspects of the language and syntax. If you have any input on that, please file an issue!

### Contributions

New contributors are welcome! Please feel free to send over a PR or file an issue.

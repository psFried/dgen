# dgen

[![Build Status](https://travis-ci.com/psFried/dgen.svg?branch=master)](https://travis-ci.com/psFried/dgen)

Generate evil test data

dgen is a CLI tool for generating pseudorandom data in arbitrary formats. The goal is to have a tool that works equally well for both textual and binary formats. Pgen is really just an interpreter for a simple domain-specific functional language. The syntax is documented [here](SYNTAX.md).

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

Pgen programs are invoked by calling `dgen run` and providing the program input in one of several ways:

- `-p`, `--program`: the program can be provided on the command line. This is easiest for simple expressions.
- `-f`, `--program-file`: interpret the given file as a program. Nice for more complex programs
- `-s`, `--stdin`: read the program from stdin

You can also add your own libraries to the program scope using the `--lib` option.

PGen has a bunch of builtin functions, too. You can list the builtin functions by executing `dgen help`. You can optionally filter the list of functions by name with `pgen help --function <name>`. Of course `pgen -h` will print out info on all of the available options.

Take a look at [the examples](dgen_examples/) for more.

### Build

dgen is built with [Rust](https://www.rust-lang.org/), and requires version 1.26 or later. Just a simple `cargo build --release` is all it takes to get a release binary. The build is currently tested only on OSX and GNU/Linux, but the intent is to start testing on Windows as well soon™️.

### Stability

This project is still in the super early stages, so major breaking changes can happen at any time. I'm still experimenting with various aspects of the language and syntax. If you have any input on that, please file an issue!

### Contributions

New contributors are welcome! Please feel free to send over a PR or file an issue.

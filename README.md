# pgen
Generate evil test data

pgen is a CLI tool for generating pseudorandom data in arbitrary formats. The goal is to have a tool that works equally well for both textual and binary formats, although textual formats are the only ones supported at the moment. Pgen is really just an interpreter for a simple domain-specific language. The syntax is documented [here](SYNTAX.md).

Pgen programs are invoked by calling `pgen run` and providing the program input in one of several ways:

- `-p`, `--program`: the program can be provided on the command line. This is easiest for simple expressions.
- `-f`, `--program-file`: interpret the given file as a program. Nice for more complex programs
- `-s`, `--stdin`: read the program from stdin

PGen has a bunch of builtin functions, too. You can list the builtin functions by executing `pgen help`. You can optionally filter the list of functions by name with `pgen help --function <name>`.





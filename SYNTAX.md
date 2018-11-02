# PGen Syntax

The PGen language syntax is intentionally as simple and minimal as possible. Most people will probably only write dgen scripts occasionally, so we want the syntax to be simple and easy to remember. As of right now, the language only has a few types. The grammar of dgen is broken down to function definitions and expressions. 

## Comments
The comment character is `#`. Everything after a `#` until the end of the line will be ignored by the interpreter.

## Expressions

Expressions are the bread and butter of any dgen program. All expressions evaluate to a "generator" that can repeatedly generate (sometimes random) data. The simplest expressions are literals, which will always return the same constant value. By using functions, it's easy to generate pseudorandom data. Expressions come in only two flavors: literals and function calls.

### Literals

Literals are the simplest type of expression there is. Each literal represents a constant expression that will always return the same value. The types of literal expressions are:

- Boolean: either `true` or `false`
- Uint: An unsigned 65 bit integer value, for example `123` or `0`
- Int: A signed 64 bit integer value, for example `-4` or `+789`. A signed int literal must always have the sign present, even for positive numbers.
- Char: A 32 bit unicode codepoint expressed as a single written character between two single-quotes, for example `'a'` or `'#'`
- String: Any valid sequence of unicode code points, surrounded by double quote characters. Example, `"foo"` or `"hello world!"`. See the notes below on Strings and Chars for more information and examples.
- Bin: A sequence of comma-separated bytes (can either by in hex or decimal notation) between two square braces, for example: `[0x04, 0xAA]` or `[]` or `[1, 2, 3]`

### Function calls

A function call takes the form of `function_name(argument1, argument2, ..., argumentN)`. For example, to generate random unsigned integers, you can call the `uint()` function. Alternatively, to generate random unsigned integers within a given range, you can call `uint(7, 33)`. The concept of flat map is also built into the dgen language as a first class citizen. Any function call may optionally include a flat mapping expression by using the syntax: `function_name(arg1, ..., argn) { value ->  <Expression> }`. Within the mapper body, `value` will always refer to the same exact value.

Expressions can be arbitrarily nested. For example, to generate random alphanumeric strings or varying lengths with double quotes around them, you could use `double_quote(alphanumeric_string(uint(3, 40)))`. This will generate strings between 3 and 40 characters long and put quotes around them.

## Function definitions

You can also define your own functions. Function definitions take the following form:
`def function_name(argument1_name: <Type>, argument2_name: <Type>, ..., argumentN_name: <Type>) = <Expression>;`

There's kind of a lot there, so let's break it down. First, all function definitions start with the keyword `def`, followed by at least one whitespace character. Then comes the function name. This is of course the name that will be used later when calling the function. After the name comes the names and types of the arguments. `<Type>` can be one of: `Boolean`, `Uint`, `Int`, `Float`, `Bin`, `Char`, or `String`. Functions that take no arguments are also valid, and just have an empty set of parentheses. After the argument list comes a single equals sign (`=`), followed by any expression. Within the body of the function, arguments can be used either by referencing their names directly (without parentheses) or calling them as functions that take no arguments. Within the body of a function, you may omit parentheses for using any arguments that were passed to your function. The end of a function definition is terminated by a mandatory semicolon (`;`).

### Function Examples

Let's create a function that repeats the string `Hello World!` a given number of times. To do this, we'll use the builtin `repeat` function.

```
# print_hello.dgen

# repeats printing "Hello World!" a bounded number of times, each on it's own line
def hello_world(min_repeats: Uint, max_repeats: Uint) = 
    repeat(uint(min_repeats, max_repeats), trailing_newline("Hello World!"));

# Calling the function will produce 4-7 lines of the text: "Hello World!"
hello_world(4, 7)
```

Here's some other examples of defining and calling functions. The following dgen program will print a series of lines like the following: `4 foos: foofoofoofoo`
```
# A simple function that always returns the string `"foo"`
def foo() = "foo";

# Repeats the string `"foo"` the given number of times
def repeat_foo(times: Uint) = repeat(times, foo());

# Makes one line of output
def make_line(num_foos: Uint) = num_foos() {foo_count ->
    concat(to_string(foo_count), " ", foo(), "s: ", repeat_foo(foo_count), "\n")
};
```

## Mapped Functions

The concept of a `flatMap` is ubuiquitous in functional programming, and the dgen language supports flat map as a first class language feature. The basic idea is that you take the value from one generator and use it create another generator. Any function call may optionally use a mapper by using the syntax:

```
<function_name>(<arguments*>) { <value> ->
    <Expression>
}
```

In a mapped function, the `<value>` will always be the same within the scope of the curly braces. This allows you to reuse the same value in multiple places instead of always generating a new one. For example, the following expression will print the an unsigned integer followed by a string of that length:

```
uint(1, 10) { string_length ->
    concat("printing a string with ", to_string(string_length), " ascii characters: ", alphanumeric_string(string_length))
}
```

Mapped functions can be used anywhere a function is called, including in the body of function definitions. For example, the following program is equivalent to the previous one:

```
# extract the above expression into a function
def my_string(len: Uint) = len() { string_length ->
    concat("printing a string with ", to_string(string_length), " ascii characters: ", alphanumeric_string(string_length))
};

# calling the function
my_string(uint(1, 10))
```

## Notes on Strings and Chars

Strings are one of the most important and complex parts of any programming language, and dgen is no exception. Strings in dgen can contain any valid sequence of unicode codepoints. Both String and Char literals can contain any unicode characters. These can either be written directly inline like `"...💩..."`, or as unidode escape sequences such as `"\U{1F4A9}"`. For a Char literal, it would look like: `'\U{1F4A9}'`. Both String and Char literals support the same escape sequences:

- `\n` for a newline
- `\r` for a carriage return
- `\t` for a tab character
- `\\` for a literal slash character
- `\u{XXXX}` can be used to insert an arbitrary unicode codepoint specified by the given hexidecimal. Neither the `u` nor the hex string are case sensitive. `\u{1F4A9}` and `\U{1f4a9}` are both equivalent.

# Modules

Each file executed by dgen is a separate module. The name of the module is the filename, minus the `.dgen` extension if one is present. Thus passing the argument `--lib foo.dgen` will result in a module named `foo` being added to the scope. Functions defined in any module may be called from any other module by using it's name directly. Take the following example:

First file
```
# in foo.dgen

def double(string: String) = string() { value ->
    concat(value, value)
};
```

Second file
```
# in bar.dgen

def double(string: String) = concat("Two times the ", string, "!");
```

When you run `dgen --lib foo.dgen --lib bar.dgen -p 'double("wat?")'` you'll get the following error:
```
Error: Compilation Error: Ambiguous function call, which could refer to multiple functions:
Called function: double(String)
Option A: double(String) - at foo.dgen:1
Option B: double(String) - at bar.dgen:1

<command line input>:1

line    1| double("wat?")
```

Within a module, it is an error to define multiple multiple functions with the same signature, but there are no restrictions on functions that are defined in separate modules. To make it clear which function you meant to call, you can always add the module name to the beginning of the function call, like `foo.double("wat?")` or `bar.double("wat?")`. If you are calling a function from the same file it's defined in, then you never need to use the module name to disambiguate it.

# More Examples

More examples can be found in the [degn_examples](dgen_examples) directory. 

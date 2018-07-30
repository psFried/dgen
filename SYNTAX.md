# PGen Syntax

The PGen language syntax is intentionally as simple and minimal as possible. Most people will probably only write pgen scripts occasionally, so we want the syntax to be simple and easy to remember. As of right now, the language only has a few types.

## Expressions

Expressions are the bread and butter of any pgen program. Expressions evaluate to something that will generate data. The simplest expressions are literals, which will always return the same constant value. By using functions, it's easy to generate pseudorandom data. Expressions come in only two flavors: literals and function calls.

### Literals

Literals are the simplest type of expression there is. Each literal represents a constant expression that will always return the same value. The types of literal expressions are:

- Boolean: either `true` or `false`
- Uint: An unsigned integer value, for example `123` or `0`
- Int: A signed integer value, for example `-4` or `+789`
- String: Any valid sequence of unicode code points, surrounded by double quote characters. Example, "foo" or "hello world!".

### Function calls

A function call takes the form of `function_name(argument1, argument2, ..., argumentN)`. Functions that take no arguments may either be called with empty parenstheses or with none at all. There is no difference. For example, to generate random unsigned integers, you can call the `uint()` function (just `uint` is also valid). Alternatively, to generate random unsigned integers within a given range, you can call `uint(7, 33)`. Passing arguments to a function will always require parentheses.


Expressions can be arbitrarily nested. For example, to generate random alphanumeric strings or varying lengths with double quotes around them, you could use `double_quote(alphanumeric_string(uint(3, 40)))`. This will generate strings between 3 and 40 characters long and put quotes around them.

## Function definitions

You can also define your own functions. Function definitions take the following form:
`def function_name(argument1_name: <Type>, argument2_name: <Type>, ..., argumentN_name: <Type>) = <Expression>;`

There's kind of a lot there, so let's break it down. First, all function definitions start with the keyword `def`, followed by at least one whitespace character. Then comes the function name. This is of course the name that will be used later when calling the function. After the name comes the names and types of the arguments. `<Type>` can be one of: `Boolean`, `Uint`, `Int`, `Float`, `Char`, or `String`. Functions that take no arguments are also valid, and just have an empty set of parentheses. After the argument list comes a single equals sign (`=`), followed by any expression. Within the body of the function, arguments can be used by calling them as functions that take no arguments. The end of a function definition is terminated by a mandatory semicolon (`;`).

### User defined function example

Let's create a function that repeats the string `Hello World!` a given number of times. To do this, we'll use the builtin `repeat` function.

```
# print_hello.pgen

# repeats printing "Hello World!" a bounded number of times, each on it's own line
def hello_world(min_repeats: Uint, max_repeats: Uint) = 
    repeat(uint(min_repeats(), max_repeats()), trailing_newline("Hello World!"));


# Calling the function
hello_world(4, 7)
```

Notice that the example also contains comments, which begin with a `#` character.

## Mapped Functions

Any function may optionally use a mapper by using the syntax:

```
function_name(arg1, ..., argn) { mapper_argument ->
    <Expression>
}
```

In a mapped function, the value of the mapper argument will always be the same within the scope of the curly braces. This allows you to reuse the same value in multiple places instead of always generating a new one. For example, the following expression will print the an unsigned integer followed by a string of that length:

```
uint(1, 10) { string_length ->
    concat("printing a string with ", string_length, " ascii characters: ", alphanumeric_string(string_length))
}
```

Mapped functions can be used anywhere a function is called, including in the body of function definitions. For example, the following program is equivalent to the previous one:

```
def my_string(len: Uint) = len() { string_length ->
    concat("printing a string with ", string_length, " ascii characters: ", alphanumeric_string(string_length))
};

my_string(uint(1, 10))
```

# Sea Guide

> ![WARNING]
> This guide contains multiple `TODO` blocks, this is because Sea is still a
> work-in-progress language! I do not recommend using it at the moment &gt;3

> ![NOTE]
> This document assumes you already have programming knowledge!

This guide's structure is based on V's documentation/guide:
<https://github.com/vlang/v/blob/master/doc/docs.md>

> I hope you're ready for a **lot** of Lord of the Rings and The Hitchhiker's
> Guide to the Galaxy references!

## Introduction

Sea is a statically typed, compiled, general-purpose programming language.

Technically it was designed with game development in mind since I made Sea with
the intent of using it in my games, however it has since grown from just that.

Sea is intentionally quite minimal, it provides you with more than C but less
than, say, C++.

Despite this minimalism, Sea is very capable. You can embed raw C code and use
any C libraries directly in Sea _without needing bindings_.

## Installation

**From Source:**

```sh
git clone https://github.com/emmathemartian/sea
cd sea
cargo build --release
cp ./target/release/sea somewhere/on/path/
```

You can also `ln -s $(pwd)/target/release/sea somewhere/on/path` if you rebuild
often and test in other places.

## Getting Started

Simply make a `main.sea` file with the following contents:

```sea
fun main(): int {
	ret 0
}
```

You can run this using `sea compile --run main.sea` (or `sea c -r main.sea`).

## Table of Contents

- [Hello, World](#hello-world)
- [Comments](#comments)
- [Blocks](#blocks)
- [Functions](#functions)
- [Symbol Visibility](#symbol-visibility)
- [Variables](#variables)
- [Types](#types)
  - [Primitives](#primitives)
  - [Strings](#strings)
  - [Characters](#characters)
  - [Numbers](#numbers)
  - [Arrays](#arrays)
    - [Multidimensional Arrays](#multidimensional-arrays)
    - [Fixed-Size Arrays](#fixed-size-arrays)
- [Modules](#modules)
- [Statements](#statements)
  - [if/else](#ifelse)
  - [switch/case](#switchcase)
  - [for](#for)
- [Expressions and Operators](#expressions-and-operators)
  - [Type Casting](#type-casting)
  - [References and Pointers](#references-and-pointers)
- [Data Types](#data-types)
  - [Records](#records)
  - [Tags](#tags)
  - [Tagged Records](#tagged-records)
  - [Type Aliases](#type-aliases)
- [Raw C Code](#raw-c-code)
- [Builtins](#builtins)
- [Documentation](#documentation)
- [Sandbox](#sandbox)
- [Pragmas](#pragmas)

## Hello World

```sea
use std/io

fun main(): int {
	println("Hello, World!")
}
```

## Comments

```sea
// Single line comment

/* Multi-line
   comment */

/* /* You can nest multi-line comments too. */ */
```

## Blocks

You're probably used to braces for blocks (`{}`), that isn't different in Sea,
but there is a shorthand you should be aware of:

```sea
fun main(): int -> ret 0
```

The `->` can be thought of as taking the next statement and wrapping it in
braces, nifty!

## Functions

```sea
fun get_gandalf(): String {
	ret "Gandalf"
}

// Functions with only one expression can be written using -> instead of {}:
// fun get_gandalf(): String -> ret "Gandalf"

fun greet(who: String) {
	print("Hello, ")
	print(who)
	println("!")
}

fun main(): int {
	greet(get_gandalf())
}
```

> ![NOTE]
> Sea does not support function overloading.

> ![WARNING] ![TODO]
> Since C does not guarantee parameter evaluation order, Sea does not either.
> This will change in the future!

## Symbol Visibility

Sea doesn't have a distinction between public and private functions. To denote
a "private" function/variable/etc, you should prefix its name with a dollar
sign (`$`).

## Variables

```sea
var ring_bearer = "Gandalf" // Mutable variable
let me = "Gandalf" // Immutable variable

ring_bearer = "Frodo"

// If you need to specify the type for a variable (i.e, calling an unbound fun):
var me_ptr: ^String = malloc(sizeof(String))

// Variable shadowing is allowed:
fun something() {
	var a = "Not a bozo"
	{
		var a = "Bozo"
		println(a) // Bozo
	}
	println(a) // Not a bozo
}
```

## Types

### Primitives

```
bool

i8 i16 i32 i64
u8 u16 u32 u64

char

f32 f64

usize

Any // equivalent to ^void (void pointer)
```

### Strings

Sea strings are a thin wrapper over character arrays. Null-termination is not
guaranteed, however strings always store their length with them.

```sea
// The string definition looks like this:
// rec String(own: bool, len: int, str: ^char)

"Hello, World!".len // == 13
```

`own` tells you weather or not the string owns the memory for its characters.
You can use this to differentiate strings from string _views_.

> ![NOTE]
> Strings defined literally **do not own their memory**. This is the case in C
> too, since strings are defined and stored in the binary, which literals
> point to.

`str` is a pointer to the first character of the C string, which you can pass
to C functions. For example:

```sea
let str = "Hello, World!"
printf(c"%.*s\n", str.len, str.str)
```

If you need to use a C string, you can use `c""` instead of `""`:

```
printf(c"Hello, World!\n")
```

### Characters

Characters can be defined using backticks (`\``):

```
`a`
`b`
`c`
`\n`
```

### Numbers

```
let the_meaning_of_life = 42 // defaults to your platform-specific integer type (typically i32)

// If you need to specify the type, you can do so like this:
let the_meaning_of_life_as_a_u8: u8 = 42

let binary_meaning_of_life = 0b101010
let hex_meaning_of_life = 0x2A

// You can use underscores (_) to make numbers look a bit more clean:
let a_big_number = 1_000_000

let a_float = 3.14192
let a_double: f64 = 3.14192
```

### Arrays

```sea
let the_fellowship = [
	"Frodo",
	"Sam",
	"Merry",
	"Pippin",
	"Aragorn",
	"Gimli",
	"Legolas"
]

// The type for the above is a `String[]`

println(the_fellowship[0]) // "Frodo"
println(the_fellowship[2]) // "Merry"

the_fellowship[0] = "Bilbo"
println(the_fellowship[0]) // "Bilbo"
```

> ![TODO]
> Make array length dynamic

#### Multidimensional Arrays

```sea
let grid = [
	[ 1, 2, 3 ],
	[ 4, 5, 6 ],
	[ 7, 8, 9 ]
]

// The type of the above is `int[][]`
```

#### Fixed-Size Arrays

```sea
let numbers: int[5] = [ 0, 1, 2, 3, 4 ]
```

## Modules

```sea
// ./library.sea
use std/io

fun library_fun() {
	println("Hello, World!")
}

// ./main.sea
use library

fun main(): int {
	library_fun()
}
```

For further organization, you can do the following:

```sea
/* File structure:
 * main.sea
 * library/
 *  lib.sea
 *  api.sea
**/

// ./library/lib.sea
let life = 42

// ./library/api.sea
fun get_life(): int -> ret life

// ./main.sea
use library/api

fun main(): int -> ret get_life()
```

`library/lib.sea` gets implicitly imported.

If you import `some/path/api`, then you'll import:

```
some/lib.sea (if exists)
some/path/lib.sea (if exists)
some/path/api.sea
```

> ![NOTE]
> Module imports are always relative to your `main.sea`. You cannot use a `..`
> in a `use` statement.

## Statements

### `if`/`else`

```sea
if 1 == 1 {
	println("A")
} else if 1 == 2 {
	println("B")
} else {
	println("C)
}

// Like functions, you can use the -> shorthand syntax:
if 1 == 1 -> println("A")
else if 1 == 2 -> println("B")
else -> println("C")
```

The only major note about if/else is that you do not use parenthesis for the
condition.

> ![TODO]
> Ternary expression

> ![WARNING] \
> `else` is not part of the `if` statement, it is its own statement.
> This means that the following code **is not valid**:
>
> ```sea
> fun do_something() -> if true { } else { }
> ```

### `switch`/`case`

```sea
let value = 0
switch value {
	case 0 -> println("Zero!")
	case 1 -> println("One!")
	case 2 -> println("Two!")
	else -> println("Something else!")
}
```

`case` will break by default, to fall through a case, you can use `fall case`:

```

let value = 0
switch value {
	fall case 0 -> println("Zero!") // will not break
	fall case 1 -> println("One!") // will not break
	case 2 -> println("Two!") // will break
	else -> println("Something else!")
}
```

### `for`

For loops in Sea have three forms:

```sea
// c-style for
for var i = 0 ; i < 10 ; i++ {
	printf(c"%d\n", i)
}

// for/in?/to
// This is an **exclusive** range, meaning that 0 is included but 10 is not
// (i.e, the range is: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9)
for 0 to 10 {
	println("Hello, World!")
}

for i in 0 to 10 {
	if i == 5 -> continue // skip `5` for the sake of showcasing `continue` statements

	printf("%d\n", i)
}

// single-expression (functionally equivalent to a while loop)
for true {
	println("Hello, World!")
	break
}

// Just like functions and if expressions, you can use -> with for loops:
for 0 to 5 -> println("Hello, World!")
```

> ![TODO]
> Finish implementation for `each/in` and document here

## Expressions and Operators

```
+     add
-     subtract
/     divide
*     multiply
not   boolean not
and   boolean and
or    boolean or
==    equals
!=    not equals
>     greater than
>=    greater than/equals
<     less than
<=    less than/equals
++    increment
--    decrement
ass   type cast
ref   reference
^     dereference
```

### Type Casting

You can cast something to another type using the `as` operator:

```sea
rec Vector2f(x: f32, y: f32)

var p = malloc(sizeof(Vector2f)) as ^Vector2f
```

### References and Pointers

You can reference a value using the `ref` expression:

```sea
var x = 0
var px = ref x

x = 10
printf("%d\n", x) // 10

// You can dereference using the `^` operator:
px^ = 5
printf("%d\n", x^) // 5
```

You'll often see `some_id^.some_other_id`, this is functionally equivalent to
C's `->` operator, which Sea does not have.

```sea
rec Name(first: String, last: String)

var n = new Name("Frodo", "Baggins")
var pn = ref n

println(pn^.first) // "Frodo"
```

To denote a pointer type, you prefix the type with `^`:

```sea
rec Person(name: String, age: int)

rec RingBearer(bearer: ^Person)

let frodo = new Person("Frodo Baggins", 33)
let ring_bearer = new RingBearer(ref frodo)
```

## Data Types

### Records

```sea
rec Name(first: String, last: String)

let name = new Name("Bilbo", "Baggins")
name.first = "Frodo"
```

Records called "structs" in many other languages. They are the exact same here,
just with a different name.

Records **do not** support inheritance. Instead, you can use a "parent" record
as a field in the "child" record:

```sea
rec Animal(name: String, says: String)

rec Feline(animal: Animal)

rec Cat(feline: Feline)

fun make_cat(): Cat -> ret new Cat(
	new Feline(
		new Animal("Cat", "meow")
	)
)
```

It's a bit messy, but it works. I plan to support a basic form of inheritance
in the future (derives).

Records also do not have methods, though this is planned.

> ![TODO]
>
> - Record derives
> - Record methods

### Tags

> ![NOTE]
> Commonly called "enums" in other languages

```sea
tag Race(
	RACE_HOBBIT
	RACE_HUMAN
	RACE_ELF
	RACE_DWARF
)

rec Person(name: String, age: int, race: Race)

let frodo = new Person("Frodo Baggins", 33, RACE_HOBBIT)

switch frodo.race {
	case RACE_HOBBIT -> println("Hobbit!")
	case RACE_HUMAN -> println("Human!")
	case RACE_ELF -> println("Elf!")
	case RACE_DWARF -> println("Dwarf!")
}
```

### Tagged Records

Tagged records (or "tagged unions") are another enumerable type, except they
can store data with them, similar to Rust enums.

```sea
tag Suit(SUIT_SPADE, SUIT_HEART, SUIT_DIAMOND, SUIT_CLUB)

rec Card(suit: Suit, value: u8)

fun print_card(card: Card) {
	switch card.value {
		case 11 -> print("Jack of ")
		case 12 -> print("Queen of ")
		case 13 -> print("King of ")
		// Sometimes aces are 1, sometimes 14
		fall case 1 {}
		case 14 -> print("Ace of ")
		// Anything else
		else -> printf(c"%d of ", card.value)
	}
	switch card.suit {
		case SUIT_SPADE -> println("Spades")
		case SUIT_HEART -> println("Hearts")
		case SUIT_DIAMOND -> println("Diamonds")
		case SUIT_CLUB -> println("Clubs")
	}
}

tag rec Move(
	Draw(),
	Play(card: Card),
	Fold(),
)

fun play(move: Move) ->
	switch move.kind {
		case Draw -> println("You drew a card!")
		case Play {
			print("You played a ")
			print_card(move.Play.card)
		}
		case Fold -> println("You folded :(")
	}

fun main(): int {
	play(new Move(Draw))
	play(new Move(Play, new Card(SUIT_SPADES, 4)))
	play(new Move(Play, new Card(SUIT_DIAMONDS, 12)))
	play(new Move(Fold))
}
```

### Type Aliases

```sea
def Integer = int

fun main(): Integer -> ret 0
```

## Hashtags

Hashtags are like modifier keywords in other languages.

```sea
#inline
fun life(): int -> ret 42

// compiles to:
// inline int life() { return 42; }
```

To define multiple hashtags, group them in parenthesis:

```sea
#(inline, static)
fun life(): int -> ret 42

// compiles to:
// static inline int life() { return 42; }
```

> **If hashtags exist, why isn't `tag rec` defined as `#tag rec`?**
>
> Hashtags don't change the syntax of the statement. `tag rec`, `rec`, and `tag`
> are all totally different syntaxes.

Here's a list of all hashtags:

```sea
// funs:
#static
#inline
#extern // currently unused, though this will be used to prevent name-mangling if/when function overloads are implemented
#noret  // marks the function with `noreturn`, use this for functions that `exit()` prematurely

// recs:
#static
#union

// defs:
#static

// tags:
#static

// tag recs:
#static
```

## Raw C Code

```sea
fun main(): int {
	raw [
		// Notice that I am not using a c-string (c""), that's because
		// this code is in a raw[] block, meaning that I am writing C!
		printf("Hello, World!\n");
	]
}

// You can also place these at top-level, like so:
raw[
#include <raylib.h>

struct Life {
	int answer;
};

Life the_meaning_of_life = (Life){ 42 };
]
```

> ![NOTE]
> Raw code has no syntax validation, safety checks, etc, this means that you
> may need to read the outputted C code to debug these!

## Builtins

I don't like builtin functions/records/etc very much so I try to use as few as
possible in Sea.

You can see all builtins in `std/lib.sea`

```sea
// Globals
var nil: ^void = NULL

// Types
def Any = ^void

def u8 = uint8_t
def u16 = uint16_t
def u32 = uint32_t
def u64 = uint64_t

def i8 = int8_t
def i16 = int16_t
def i32 = int32_t
def i64 = int64_t

def f32 = float
def f64 = double

def usize = size_t

// Records
rec String(own: bool, len: int, str: ^char)
```

Since Sea has no namespaces (\*yet), functions imported from other modules are
thrown into the global scope :/

## Documentation

You can write documentation comments ("doc comments") using the following two
syntaxes:

```sea
/// Returns <nil>.
fun get_nil(): Any -> ret nil

/**
 * This function returns <'it>.
 *
 * Args:
 *  it: Any - The pointer to return.
**/
fun something(it: Any): Any -> ret it
```

Doc comments are written in a markdown-style format:

```
`text`     - code
*text*     - italic
**text**   - bold
***text*** - italic/bold
<text>     - link to a function, record, etc
<'text>    - link to a parameter or argument
```

To write "good" doc comments, we **recommend** the following guidelines:

> Ultimately it is up to _you_, the programmer, to decide what you use. Though,
> for public libraries, I highly recommend using these guidelines!

> These guidelines are based on the Javadoc guidelines, since I'm quite used to
> that style: <https://www.oracle.com/technical-resources/articles/java/javadoc-tool.html#styleguide>

- Don't exceed 90 characters in length for any given line. Use a line break on
  the last word that is less than or equal to 90 characters.

```sea
/**
* A really really long line that rambles about something with the intention of describing  <-- This line is right at 90 characters, so we break now
* a function.
**/

/**
 * Another really really long line that rambles about something with the intention of <-- Including "describing" on this line exceeds 90 characters, so it will be put on the next line.
 * describing a function.
**/
```

- Use 3rd person, not 2nd person:

```sea
/// Return nil. <-- Bad
/// Returns nil. <-- Good
```

- Use periods to end sentences:

```sea
/// Returns nil <-- Bad
/// Returns nil. <-- Good
```

- Function docs should typically begin with a verb:

```sea
/// This function returns nil. <-- Bad
/// Returns nil. <-- Good
```

- The usage of `aka` ("also known as"), `i.e.` ("that is" or "to be specific"),
  and `e.g.` ("for example") is okay! Avoid other less-known abbreviations
  though.

## Sandbox

See [sandbox.md](./sandbox.md), there's a lot to the sandbox :P

## Pragmas

Pragmas allow you to control the Sea compiler from within your Sea code. You
shouldn't have to use these often, although if you're interacting with external
C libraries, you may need to.

Pragmas use the `pragma` keyword and look like this:

```sea
// Adds `-lraylib` to the C compiler's flags when compiling your code
pragma add_cc_flag("-lraylib")

// This is equivalent to the above pragma
pragma add_library("raylib")
```

Pragmas may look like function invocations, and whilst similar, they fulfil a
vastly different role.

> ![NOTE]
> Pragmas cannot be user-defined, nor are they macros!

Here's a list of all pragmas:

```sea
add_cc_flag(flag: String) // Add a flag to the C compiler
add_library(link: String) // Add a link (`-l`) to CC flags
add_include_dir(dir: String) // Add an include directory to CC flags, relative to the current file's directory
```

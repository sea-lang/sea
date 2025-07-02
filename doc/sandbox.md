# Sandbox

The sandbox is a fairly niche feature in Sea, it's similar to a REPL but not
quite the same thing.

> Explaining the sandbox is a bit difficult, so the docs here will take the form
> of a tutorial more than a reference guide.

To open the sandbox, use `sea sandbox` (or `sea s`). This will present you with
a pretty underwhelming prompt:

```
[1]:
```

In this prompt, type the following (**all on one line**):

```
[1]: fun main(): int -> ret 0
```

After pressing enter, you'll notice that... pretty much nothing happens:

```
[1]: fun main(): int -> ret 0
[2]:
```

Let's try something a bit more interesting, input `\exec` and press enter:

```
[1]: fun main(): int -> ret 0
[2]: \exec
: Compiling Sea
: Compiling C: tcc -g3 -o .sea/sandbox/program .sea/sandbox/program.c
: Executing: .sea/sandbox/program
[2]:
```

Well that did something! This executed the function we just wrote!

The sandbox has "commands" which all start with a backslash. To list each of
them, use `\help`.

Put simply, the sandbox gives you a quick place to write scratch code. Though it
can do much more than just this, here's a bit more:

```
[1]: fun main(): int -> ret 0
[2]: \exec
: Compiling Sea
: Compiling C: tcc -g3 -o .sea/sandbox/program .sea/sandbox/program.c
: Executing: .sea/sandbox/program
[2]: \replace
Replacing lines.
[2]: \pos 1
[1]: \pause
Paused compilation.
[P 1]: fun main(): int {
[P 2]: printf(c"Hello, World!\n")
[P 3]: }
[P 4]: \pos 1
[P 1]: \replace
Shifting lines.
[P 1]: raw[ #include <stdio.h> ]
[P 2]: \pos end
[P 5]: \pause
Unpaused compilation.
[5]: \exec
: Compiling Sea
: Compiling C: tcc -g3 -o .sea/sandbox/program .sea/sandbox/program.c
: Executing: .sea/sandbox/program
Hello, World!
[5]:
```

Woah okay that was a lot! Let me explain what's going on here:

Firstly, `\replace` is a command which toggles between what should happen when
you are editing a line _before_ the current one. By default, lines get shifted,
meaning if you do this:

```
[1]: fun main(): int -> ret 0
[2]: \pos 1
[1]: fun main(): int -> ret 1
```

Then the Sea code would look like this:

```sea
fun main(): int -> ret 1
fun main(): int -> ret 0 // This line got shifted down
```

To instead replace the line, we toggle `\replace` on.

You may have guessed what `\pos` does. It jumps to the provided line! You can
also use `\pos end` to jump to the end of the file.

Next there's `\pause`. Using this toggles auto-compilation. By default, any time
you input a line, the sandbox will attempt to compile your code. If you are
writing "partial" statements then that's an issue since you'll be compiling code
with invalid syntax. To mitigate this, we disable auto-compilation with the
command.

Next we jump back to the beginning and enable line-shifting so that we can
#include `stdio.h` for the `printf` function, then we jump back to the end using
`\pos end`.

Finally we unpause compilation and execute, presenting `Hello, World!` :D

## Command Reference

(from `\help`)

```
Sandbox Commands:
  \h \help            - Show this message
  \e \echo            - Echo all lines
  \q \quit            - Quits
  \R \reset           - Reset the sandbox
  \p \pos <int|"end"> - Jump to the given line (int) or to the last line ("end")
  \u \pause           - Pause auto-compilation
  \r \read <file>     - Read from a given file
  \w \write <file>    - Write to a given file
  \P \replace         - Toggle line replacement when editing previous lines, when off, lines are shifted (default: off)
  \a \ast             - Print the AST for the sandbox's code
  \x \exec <args>     - Execute the sandbox's code using the provided arguments, if any
  \A \autoexec        - Toggle automatic execution after compilation (default: off)
  \R \args <args>     - Set args to pass to the program when no others are provided
```

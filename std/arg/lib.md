The `arg` Sea module provides a simple argument parser, here are some examples
of what it supports.

> ![WARNING]
> Things here may change! I'm probably going to opinionate it for my use cases
> and move it to not be part of the standard library.

```sh
program -a    # a short flag
program --all # a long flag

program -n Gandalf      # a short option using a space delimiter
program -nGandalf       # a short option using no delimiter
program -n=Gandalf      # a short option using `=`
program --named Gandalf # a long option using a space delimiter
program --named=Gandalf # a long option using `=`

program Frodo # a positional argument

program Frodo Bilbo # another positional argument

program -an Gandalf # grouped flags and options with a space delimiter
program -anGandalf  # grouped flags and options with no delimiter
program -an=Gandalf # grouped flags and options with `=`

program -a Frodo -n Gandalf # a short flag (-a), a short option (-n Gandalf), and a positional argument (Frodo)

program -an Gandalf -- Frodo # a short flag (-a), a short option (-n Gandalf), and a positional argument (Frodo)
program -an Gandalf -- Frodo -b # a short flag (-a), a short option (-n Gandalf), and two positional arguments (Frodo, -b)
```

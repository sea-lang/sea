# Reef Specification

Reef is a tiny solely-key/value config format made for usage for Sea configs.

It's a valid subset of TOML, meaning any existing TOML parser should be able to
parse Reef perfectly.

...and a subset of Java `.properties`

....oh and `.ini`

.....and a handful of others

Reef is designed to be easy to parse, easy to implement, and easy to use. Though
that also means it's more limited in its capabilities, which is intentional.

```reef
# Comments can only be placed on lines by themselves and must use a hashtag (#)

# Keys are alphanumeric (a-z, A-Z, 0-9, and _)
key = "value"

# Valid values are strings, numbers, and booleans
string = "String"
number = 42
boolean = true

# Reef does not have lists or maps. For lists, you can use a character-delimited string
# This is not all that rare either, most shells use a semicolon-delimited string for $PATH
fake_list = "value;another value;something else"

# As for maps, you can "categorize" the key
module_name = "Some Module"
module_author = "Gandalf"
```

## Why didn't you just use &lt;x&gt;?

1. I dislike most existing config file formats.
2. Basic key-value config formats are _extremely_ simple to parse and don't
   require users to learn anything new. 99% of programmers will understand
   `key = value` :P
3. It's also really easy to repurpose a TOML parser into a Reef parser, making
   it even easier to implement since you don't even _have_ to implement it!

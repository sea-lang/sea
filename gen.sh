#!/usr/bin/env sh
set -e
cd proto/syntax
sh clean.sh
antlr4 -v 4.13.2 -Dlanguage=Python3 Lexer.g4
antlr4 -v 4.13.2 -Dlanguage=Python3 Parser.g4

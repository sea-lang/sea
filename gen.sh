#!/usr/bin/env sh
set -e
cd src/syntax
sh clean.sh
antlr4 -Dlanguage=Python3 Lexer.g4
antlr4 -Dlanguage=Python3 Parser.g4

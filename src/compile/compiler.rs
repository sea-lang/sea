use std::{fs::File, path::PathBuf, process::exit};

use crate::parse::{ast::Node, parser::Parser};

use super::{
    error::CompilerError,
    symbol::{Symbol, SymbolTable},
};

pub struct Compiler<'a> {
    pub output_path: PathBuf,
    pub output_file: File,
    pub scope: usize,
    pub symbols: SymbolTable,
    pub parser: Parser<'a>,
}

impl<'a> Compiler<'a> {
    pub fn new(output_path: PathBuf, output_file: PathBuf, parser: Parser<'a>) -> Self {
        Compiler {
            output_path,
            output_file: File::create(output_file).unwrap(),
            scope: 0,
            symbols: SymbolTable::new(),
            parser,
        }
    }

    pub fn throw_exception(&self, error: CompilerError, help: Option<&str>, node: Node) -> ! {
        println!(
            "\x1b[31;1m{}:{}:{}:\x1b[0;1m {error}\x1b[0m",
            self.parser.lexer.file.to_str().unwrap(),
            node.line,
            node.column
        );

        let lines = self.parser.lexer.get_lines(node.line);
        if lines.len() == 0 {
            println!("No line information available :(");
            println!("This error shouldn't happen, please report it.");
            println!("Debug: node={}", node.clone())
        } else {
            // Determine the longest integer by digit so that we can make our error prettier.
            let longest_length = lines.iter().map(|it| it.0).max().unwrap().to_string().len();

            for (line_index, line_str) in lines {
                // We replace `\t` with ` ` so that no matter the terminal indentation, the underline will be aligned
                // let indents = line_str.chars().filter(|it| *it == '\t').count();
                let sanitized = line_str.replace('\t', "    ");

                println!("\x1b[1;34m{line_index:>longest_length$} | \x1b[0m{sanitized}");

                //TODO: Implement an underline?
            }
        }

        if let Some(help) = help {
            println!("\x1b[1;32mhelp:\x1b[0m {}", help);
        }

        exit(1)
    }

    pub fn push_scope(&mut self) {
        self.scope += 1;
    }

    pub fn pop_scope(&mut self) {
        self.symbols.remove_symbols_from_scopes(self.scope);
        self.scope -= 1;
    }

    pub fn add_fun(&mut self, name: String) {
        self.symbols.add_symbol(name, Symbol::Fun);
    }

    pub fn add_rec(&mut self, name: String) {
        self.symbols.add_symbol(name, Symbol::Rec);
    }

    pub fn add_def(&mut self, name: String) {
        self.symbols.add_symbol(name, Symbol::Def);
    }

    pub fn add_mac(&mut self, name: String) {
        self.symbols.add_symbol(name, Symbol::Mac);
    }

    pub fn add_tag(&mut self, name: String) {
        self.symbols.add_symbol(name, Symbol::Tag);
    }

    pub fn add_tag_rec(&mut self, name: String) {
        self.symbols.add_symbol(name, Symbol::TagRec);
    }

    pub fn add_var(&mut self, name: String) {
        self.symbols.add_symbol(name, Symbol::Var);
    }
}

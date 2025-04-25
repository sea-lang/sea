use std::{fs::File, path::PathBuf, process::exit};

use crate::{
    parse::{ast::Node, parser::Parser},
    util,
};

use super::{
    error::CompilerError,
    symbol::{Symbol, SymbolTable},
    type_::SeaType,
};

pub struct Compiler<'a> {
    pub output_path: PathBuf,
    pub output_file: File,
    pub libpaths: Vec<PathBuf>,
    pub scope: usize,
    pub symbols: SymbolTable,
    pub parser: Parser<'a>,
    pub usages: Vec<PathBuf>,
    pub module_stack: Vec<PathBuf>,
    pub file_stack: Vec<PathBuf>,
}

impl<'a> Compiler<'a> {
    pub fn new(
        output_path: PathBuf,
        output_file: PathBuf,
        libpaths: Vec<PathBuf>,
        parser: Parser<'a>,
    ) -> Self {
        Compiler {
            output_path,
            output_file: File::create(output_file).unwrap(),
            libpaths,
            scope: 0,
            symbols: SymbolTable::new(),
            parser,
            usages: vec![],
            module_stack: vec![PathBuf::from("main")],
        }
    }

    pub fn throw_exception(
        &self,
        source_file: String,
        error: CompilerError,
        help: Option<&str>,
        node: Node,
    ) -> ! {
        println!(
            "\x1b[31;1m{source_file}:{}:{}:\x1b[0;1m {error}\x1b[0m",
            node.line, node.column
        );

        let lines = util::get_lines_from(&source_file, node.line);
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

    // Get the paths to each file in a given module
    pub fn get_use_paths(
        &mut self,
        path: PathBuf,
        selections: Option<Vec<String>>,
    ) -> Result<Vec<PathBuf>, String> {
        let mut paths: Vec<PathBuf> = vec![];

        for libpath in &self.libpaths {
            let p = libpath.join(&path);
            if p.exists() && p.is_dir() {
                if let Some(selections) = selections {
                    // Check if lib.sea exists, if so we'll import that first
                    if p.join("lib.sea").exists() {
                        paths.push(p.join("lib.sea"))
                    }

                    for s in selections {
                        if s == "lib" {
                            continue; // Handled above
                        }

                        let file_path = p.join(s);

                        if file_path.with_extension("sea").is_file() {
                            paths.push(file_path.clone());
                        } else if file_path.is_dir() {
                            match self.get_use_paths(file_path, None) {
                                Ok(mut p) => paths.append(&mut p),
                                Err(err) => return Err(err),
                            }
                        } else {
                            return Err(format!("{file_path:?} is not a valid module"));
                        }
                    }
                } else {
                    // Check if lib.sea exists, if so we'll import that first
                    if p.join("lib.sea").exists() {
                        paths.push(p.join("lib.sea"))
                    }

                    // Import each file in the module
                    for file in p.read_dir().unwrap() {
                        let file_path = file.unwrap().path();

                        if file_path.file_name().unwrap() == "lib.sea" {
                            continue; // Handled above
                        }

                        if file_path.is_file() && file_path.extension().unwrap() == "sea" {
                            paths.push(file_path.clone());
                        }
                    }
                }

                self.usages.push(p);
                return Ok(paths);
            }
        }

        Err(format!("no such module: {path:?}"))
    }

    pub fn add_fun(&mut self, name: String, params: Vec<SeaType>, rets: SeaType) {
        self.symbols.add_symbol(name, Symbol::Fun { params, rets });
    }

    pub fn add_rec(&mut self, name: String, fields: Vec<(String, SeaType)>) {
        self.symbols.add_symbol(name, Symbol::Rec { fields });
    }

    pub fn add_def(&mut self, name: String, typ: SeaType) {
        self.symbols.add_symbol(name, Symbol::Def { typ });
    }

    pub fn add_mac(&mut self, name: String) {
        self.symbols.add_symbol(name, Symbol::Mac);
    }

    pub fn add_tag(&mut self, name: String, entries: Vec<String>) {
        self.symbols.add_symbol(name, Symbol::Tag { entries });
    }

    pub fn add_tag_rec(&mut self, name: String, entries: Vec<(String, Vec<(String, SeaType)>)>) {
        self.symbols.add_symbol(name, Symbol::TagRec { entries });
    }

    pub fn add_var(&mut self, name: String, typ: SeaType, mutable: bool) {
        self.symbols
            .add_scoped_symbol(name, self.scope, Symbol::Var { typ, mutable });
    }
}

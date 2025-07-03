use std::{fs::File, path::PathBuf, process::exit};

use crate::{
    hashtags::{DefTags, FunTags, RecTags, TagRecTags, TagTags},
    parse::{ast::Node, parser::Parser},
    util,
};

use super::{
    error::CompilerError,
    pragmas::Pragma,
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
    pub file_stack: Vec<PathBuf>,
    pub cc_flags: Vec<String>,
}

impl<'a> Compiler<'a> {
    pub fn new(
        output_path: PathBuf,
        output_file: PathBuf,
        libpaths: Vec<PathBuf>,
        parser: Parser<'a>,
    ) -> Self {
        let p = parser.lexer.file.clone();
        Compiler {
            output_path,
            output_file: File::create(output_file).unwrap(),
            libpaths,
            scope: 0,
            symbols: SymbolTable::new(),
            parser,
            usages: vec![],
            file_stack: vec![p],
            cc_flags: vec![],
        }
    }

    pub fn throw(&self, error: CompilerError, help: Option<&str>, node: Node) -> ! {
        let source_file = self.file_stack.last().unwrap().to_str().unwrap();

        println!(
            "\x1b[31;1m{source_file}:{}:{}:\x1b[0;1m {error}\x1b[0m",
            node.line, node.column
        );

        let lines = util::get_lines_from_file(&source_file, node.line);
        if lines.len() == 0 {
            println!("No line information available :(");
            println!("This error shouldn't happen, please report it.");
            println!("Debug: source_file={source_file}, node={}", node.clone())
        } else {
            // Determine the longest integer by digit so that we can make our error prettier.
            let longest_length = lines.iter().map(|it| it.0).max().unwrap().to_string().len();

            for (line_index, line_str) in lines {
                // We replace `\t` with ` ` so that no matter the terminal indentation, the underline will be aligned
                let indents = line_str.chars().filter(|it| *it == '\t').count();
                let sanitized = line_str.replace('\t', "    ");

                println!("\x1b[1;34m{line_index:>longest_length$} | \x1b[0m{sanitized}");
                if line_index == node.line {
                    // Determine the column that the node is on to highlight it
                    println!(
                        "\x1b[1;34m{} | {}\x1b[31m^\x1b[0m",
                        " ".repeat(longest_length),
                        " ".repeat(node.column - 1 - indents + (indents * 4)),
                    );
                }
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

    pub fn uses(&self, path: &PathBuf) -> bool {
        self.usages.contains(path)
    }

    // Get the paths to each file in a given module
    pub fn get_use_paths(&mut self, path: PathBuf) -> Result<Vec<PathBuf>, String> {
        if self.uses(&path) {
            return Ok(vec![]);
        }

        let mut paths: Vec<PathBuf> = vec![];

        for libpath in &self.libpaths {
            let p = libpath.join(&path);

            if !p.exists() || p.is_file() {
                continue;
            }

            // Check if lib.sea exists, if so we'll import that first
            if p.join("lib.sea").exists() && !self.uses(&p.join("lib.sea")) {
                paths.push(p.join("lib.sea"))
            }

            // Import each file in the module
            for file in p.read_dir().unwrap() {
                let file_path = file.unwrap().path();

                if file_path.file_name().unwrap() == "lib.sea" {
                    continue; // Handled above
                }

                if self.uses(&file_path) {
                    continue;
                }

                if file_path.is_file() && file_path.extension().unwrap() == "sea" {
                    paths.push(file_path.clone());
                }
            }

            return Ok(paths);
        }

        Err(format!("no such module: {path:?}"))
    }

    pub fn add_fun(
        &mut self,
        name: String,
        tags: Vec<FunTags>,
        params: Vec<SeaType>,
        rets: SeaType,
    ) {
        self.symbols
            .add_symbol(name, Symbol::Fun { tags, params, rets });
    }

    pub fn add_rec(&mut self, name: String, tags: Vec<RecTags>, fields: Vec<(String, SeaType)>) {
        self.symbols.add_symbol(name, Symbol::Rec { tags, fields });
    }

    pub fn add_def(&mut self, name: String, tags: Vec<DefTags>, typ: SeaType) {
        self.symbols.add_symbol(name, Symbol::Def { tags, typ });
    }

    pub fn add_tag(&mut self, name: String, tags: Vec<TagTags>, entries: Vec<String>) {
        self.symbols.add_symbol(name, Symbol::Tag { tags, entries });
    }

    pub fn add_tag_rec(
        &mut self,
        name: String,
        tags: Vec<TagRecTags>,
        entries: Vec<(String, Vec<(String, SeaType)>)>,
    ) {
        self.symbols
            .add_symbol(name, Symbol::TagRec { tags, entries });
    }

    pub fn add_var(&mut self, name: String, typ: SeaType, mutable: bool) {
        self.symbols
            .add_scoped_symbol(name, self.scope, Symbol::Var { typ, mutable });
    }

    fn format_pragma_string(&self, s: String) -> String {
        s.replace(
            "${dir}",
            self.file_stack
                .last()
                .unwrap()
                .parent()
                .unwrap()
                .to_str()
                .unwrap(),
        )
    }

    pub fn handle_pragma(&mut self, node: Node) {
        let pragma = Pragma::from_node(&node).unwrap_or_else(|it| self.throw(it, None, node));
        match pragma {
            Pragma::AddCCFlag(it) => self.cc_flags.push(self.format_pragma_string(it)),
            Pragma::AddLibrary(it) => self
                .cc_flags
                .push(format!("-l{}", self.format_pragma_string(it))),
            Pragma::AddIncludeDir(it) => self
                .cc_flags
                .push(format!("-I{}", self.format_pragma_string(it))),
        }
    }
}

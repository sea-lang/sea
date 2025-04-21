use std::{fs::File, path::PathBuf};

use super::symbol::SymbolTable;

pub struct Compiler {
    pub output_path: PathBuf,
    pub output_file: File,
    pub scope: usize,
    pub symbols: SymbolTable,
}

impl Compiler {
    pub fn push_scope(&mut self) {
        self.scope += 1
    }

    pub fn pop_scope(&mut self) {
        self.scope -= 1
    }
}

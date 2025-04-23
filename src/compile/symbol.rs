use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Fun,
    Rec,
    Def,
    Mac,
    Tag,
    TagRec,
    Var,
}

impl Symbol {
    pub fn instantiatable(&self) -> bool {
        self == &Symbol::Rec || self == &Symbol::TagRec
    }

    pub fn invocable(&self) -> bool {
        self == &Symbol::Fun || self == &Symbol::Var //TODO: Check if variable type is function pointer
    }
}

pub struct SymbolTable {
    symbols: HashMap<String, (usize, Symbol)>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: String, kind: Symbol) {
        self.add_scoped_symbol(symbol, 0, kind);
    }

    pub fn add_scoped_symbol(&mut self, symbol: String, scope: usize, kind: Symbol) {
        self.symbols.insert(symbol, (scope, kind));
    }

    pub fn remove_symbol(&mut self, symbol: String) {
        self.symbols.remove(&symbol);
    }

    // Removes all symbols from scopes deeper or in the provided `scope` index.
    pub fn remove_symbols_from_scopes(&mut self, scope: usize) {
        let _ = self.symbols.iter_mut().filter(|(_, (s, _))| *s < scope);
    }

    pub fn get_symbol(&self, symbol: String) -> Option<&Symbol> {
        if self.symbols.contains_key(&symbol) {
            Some(&self.symbols[&symbol].1)
        } else {
            None
        }
    }
}

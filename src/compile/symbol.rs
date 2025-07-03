use std::collections::HashMap;

use crate::hashtags::{DefTags, FunTags, RecTags, TagRecTags, TagTags};

use super::type_::SeaType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    Fun {
        tags: Vec<FunTags>,
        params: Vec<SeaType>,
        rets: SeaType,
    },
    Rec {
        tags: Vec<RecTags>,
        fields: Vec<(String, SeaType)>,
    },
    Def {
        tags: Vec<DefTags>,
        typ: SeaType,
    },
    Tag {
        tags: Vec<TagTags>,
        entries: Vec<String>,
    },
    TagRec {
        tags: Vec<TagRecTags>,
        entries: Vec<(String, Vec<(String, SeaType)>)>,
    },
    Var {
        typ: SeaType,
        mutable: bool,
    },
}

impl Symbol {
    pub fn instantiatable(&self) -> bool {
        match self {
            Symbol::Rec { tags: _, fields: _ } => true,
            Symbol::TagRec {
                tags: _,
                entries: _,
            } => true,
            _ => false,
        }
    }

    pub fn invocable(&self) -> bool {
        match self {
            Symbol::Fun {
                tags: _,
                params: _,
                rets: _,
            } => true,
            Symbol::Var { typ, mutable: _ } => typ.funptr_rets.is_some(),
            _ => false,
        }
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

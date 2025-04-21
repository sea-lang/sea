pub enum Symbol {
    Fun,
    Rec,
    Tag,
    TagRec,
    Mac,
    Var,
    Let,
    FunParam,
}

pub struct SymbolTable {}

impl SymbolTable {
    pub fn add_symbol(&mut self, symbol: String, kind: Symbol) {}

    pub fn add_scoped_symbol(&mut self, symbol: String, scope: usize, kind: Symbol) {}
}

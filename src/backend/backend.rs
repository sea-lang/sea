use crate::parse::ast::Node;

pub trait Backend {
    fn write(&mut self, node: Node);
}

use crate::parse::ast::{Node, NodeKind};

use super::{compiler::Compiler, type_::SeaType};

pub fn infer_type_of_node(compiler: Compiler, node: Node) -> Option<SeaType> {
    Some(match node.node {
        NodeKind::ExprGroup(node) => infer_type_of_node(compiler, *node)?,
        NodeKind::ExprNumber(_) => todo!(),
        NodeKind::ExprString(_) => todo!(),
        NodeKind::ExprCString(_) => todo!(),
        NodeKind::ExprChar(_) => todo!(),
        NodeKind::ExprTrue => todo!(),
        NodeKind::ExprFalse => todo!(),
        NodeKind::ExprIdentifier(_) => todo!(),
        NodeKind::ExprBlock(nodes) => todo!(),
        NodeKind::ExprNew { id, params } => todo!(),
        NodeKind::ExprUnaryOperator { kind, value } => todo!(),
        NodeKind::ExprBinaryOperator { kind, left, right } => todo!(),
        NodeKind::ExprInvoke { left, params } => todo!(),
        NodeKind::ExprMacInvoke { name, params } => todo!(),
        NodeKind::ExprList(nodes) => todo!(),
        NodeKind::ExprVar { name, typ, value } => todo!(),
        NodeKind::ExprLet { name, typ, value } => todo!(),
        _ => return None,
    })
}

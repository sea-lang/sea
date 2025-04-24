use crate::parse::ast::{Node, NodeKind};

use super::{compiler::Compiler, symbol, type_::SeaType};

pub fn infer_type_of_node(compiler: &Compiler, node: &Node) -> Option<SeaType> {
    Some(match &node.node {
        NodeKind::ExprGroup(node) => infer_type_of_node(compiler, &node)?,
        NodeKind::ExprNumber(value) => {
            if value.contains('.') {
                SeaType::F32.clone()
            } else {
                SeaType::I32.clone()
            }
        }
        NodeKind::ExprString(_) => SeaType::STRING.clone(),
        NodeKind::ExprCString(_) => SeaType::C_STRING.clone(),
        NodeKind::ExprChar(_) => SeaType::CHAR.clone(),
        NodeKind::ExprTrue => SeaType::BOOL.clone(),
        NodeKind::ExprFalse => SeaType::BOOL.clone(),
        NodeKind::ExprIdentifier(id) => match compiler.symbols.get_symbol(id.clone()) {
            Some(symbol) => match symbol {
                symbol::Symbol::Var { typ, mutable: _ } => typ.clone(),
                _ => return None,
            },
            None => return None,
        },
        NodeKind::ExprBlock(nodes) => todo!(),
        NodeKind::ExprNew {
            id,
            params: _params,
        } => SeaType::named_type(&id),
        NodeKind::ExprUnaryOperator { kind: _kind, value } => infer_type_of_node(compiler, &value)?,
        NodeKind::ExprBinaryOperator {
            kind: _kind,
            left,
            right: _right,
        } => infer_type_of_node(compiler, &left)?,
        NodeKind::ExprInvoke { left, params: _ } => match &left.node {
            NodeKind::ExprIdentifier(id) => {
                let sym = compiler.symbols.get_symbol(id.clone()).unwrap();
                match sym {
                    symbol::Symbol::Fun { params: _, rets } => rets.clone(),
                    symbol::Symbol::Var { typ, mutable: _ } => {
                        if typ.funptr_rets.is_some() {
                            return Some(typ.funptr_rets.as_ref().unwrap().as_ref().clone());
                        } else {
                            return None;
                        }
                    }
                    _ => return None,
                }
            }
            _ => todo!(),
        },
        NodeKind::ExprMacInvoke { name, params } => todo!(),
        NodeKind::ExprList(nodes) => {
            if nodes.len() == 0 {
                return None;
            } else {
                infer_type_of_node(compiler, &nodes[0])?.array_of_size(nodes.len())
            }
        }
        NodeKind::ExprVar { name, typ, value } => todo!(),
        NodeKind::ExprLet { name, typ, value } => todo!(),
        _ => return None,
    })
}

use crate::parse::{
    ast::{Node, NodeKind},
    operator::OperatorKind,
};

use super::{compiler::Compiler, symbol, type_::SeaType};

pub fn infer_type_of_node(compiler: &Compiler, node: &Node) -> Result<SeaType, String> {
    // println!("inferring type of:");
    // node.pretty_print();

    Ok(match &node.node {
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
                _ => {
                    return Err(format!(
                        "cannot infer type for non-variable symbol: {symbol:?}"
                    ))
                }
            },
            _ => return Err(format!("symbol undefined or unbound: {id}")),
        },
        NodeKind::ExprBlock(_) => return Err("cannot infer type for block expressions".to_string()),
        NodeKind::ExprNew {
            id,
            params: _params,
        } => SeaType::named_type(&id),
        NodeKind::ExprUnaryOperator { kind, value } => match *kind {
            OperatorKind::Ref => infer_type_of_node(compiler, value)?.pointer(),
            OperatorKind::Deref => infer_type_of_node(compiler, value)?.unpointer(),
            _ => infer_type_of_node(compiler, value)?,
        },
        NodeKind::ExprBinaryOperator { kind, left, right } => match kind {
            OperatorKind::Dot => {
                // The right side of a dot operator will always be an identifier
                let id = match &right.node {
                    NodeKind::ExprIdentifier(id) => id,
                    _ => unreachable!(),
                };

                let typ = infer_type_of_node(compiler, left)?;
                let name = typ.name;
                let sym = compiler.symbols.get_symbol(name.clone());
                if sym.is_none() {
                    return Err(format!("struct not found: {name}"));
                }
                let sym = sym.unwrap();
                match sym {
                    symbol::Symbol::Rec { tags: _, fields } => {
                        for (field_name, field_type) in fields {
                            if *field_name == *id {
                                return Ok(field_type.clone());
                            }
                        }
                        return Err(format!("struct `{name}` has no field `{id}`"));
                    }
                    symbol::Symbol::TagRec {
                        tags: _,
                        entries: _,
                    } => SeaType {
                        pointers: 0,
                        name: format!("_{name}_{id}"),
                        arrays: vec![],
                        funptr_args: None,
                        funptr_rets: None,
                    },
                    _ => {
                        return Err(format!(
                            "type of left side of dot (`.`) operator does not exist"
                        ))
                    }
                }
            }
            OperatorKind::As => SeaType::from_node(right.as_ref().clone()).unwrap(),
            OperatorKind::Assign => infer_type_of_node(compiler, right)?,
            _ => infer_type_of_node(compiler, left)?,
        },
        NodeKind::ExprInvoke { left, params: _ } => match &left.node {
            NodeKind::ExprIdentifier(id) => {
                let sym = compiler.symbols.get_symbol(id.clone());
                match sym {
                    Some(sym) => match sym {
                        symbol::Symbol::Fun {
                            tags: _,
                            params: _,
                            rets,
                        } => rets.clone(),
                        symbol::Symbol::Var { typ, mutable: _ } => {
                            if typ.funptr_rets.is_some() {
                                return Ok(typ.funptr_rets.as_ref().unwrap().as_ref().clone());
                            } else {
                                return Err(format!(
                                    "cannot invoke a non-function pointer variable: {id}"
                                ));
                            }
                        }
                        _ => {
                            return Err(format!(
                                "cannot invoke a non-function (or function pointer): {node}"
                            ))
                        }
                    },
                    _ => return Err(format!("no such identifier: {id}")),
                }
            }
            _ => todo!(),
        },
        NodeKind::ExprList(nodes) => {
            if nodes.len() == 0 {
                return Err("cannot infer type of empty list".to_string());
            } else {
                infer_type_of_node(compiler, &nodes[0])?.array_of_size(nodes.len())
            }
        }
        NodeKind::ExprVar {
            name: _,
            typ: _,
            value: _,
        } => todo!(),
        NodeKind::ExprLet {
            name: _,
            typ: _,
            value: _,
        } => todo!(),
        _ => return Err(format!("cannot infer type for node {node}")),
    })
}

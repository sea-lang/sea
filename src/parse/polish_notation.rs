// Converts an operator node into Polish Notation for debugging purposes

use core::fmt;

use super::{
    ast::{Node, NodeKind},
    operator::OperatorKind,
};

const RESET: &'static str = "\x1b[0m";
const NAME: &'static str = "\x1b[34m";
const VALUE: &'static str = "\x1b[31m";

pub enum PolishNodeTree {
    Leaf(String),
    Branch(String, Vec<PolishNodeTree>),
}

impl PolishNodeTree {
    pub fn from_node_vec(nodes: Vec<Node>) -> Option<Vec<Self>> {
        let mut tree_nodes: Vec<PolishNodeTree> = vec![];
        for node in nodes {
            match PolishNodeTree::from_node(node) {
                Some(it) => tree_nodes.push(it),
                None => return None,
            }
        }
        Some(tree_nodes)
    }

    pub fn from_node(node: Node) -> Option<Self> {
        Some(match node.node {
            NodeKind::ExprGroup(node) => PolishNodeTree::Branch(
                "group".to_string(),
                vec![PolishNodeTree::from_node(*node.clone()).unwrap()],
            ),
            NodeKind::ExprNumber(value) => PolishNodeTree::Leaf(value),
            NodeKind::ExprString(value) => PolishNodeTree::Leaf(value),
            NodeKind::ExprChar(value) => PolishNodeTree::Leaf(value.to_string()),
            NodeKind::ExprTrue => PolishNodeTree::Leaf("true".to_string()),
            NodeKind::ExprFalse => PolishNodeTree::Leaf("false".to_string()),
            NodeKind::ExprIdentifier(value) => PolishNodeTree::Leaf(value),
            NodeKind::ExprBlock(_) => PolishNodeTree::Leaf("block".to_string()),
            NodeKind::ExprNew { id, params } => {
                let mut nodes = PolishNodeTree::from_node_vec(params).unwrap();
                nodes.insert(0, PolishNodeTree::Leaf(id));
                PolishNodeTree::Branch("new".to_string(), nodes)
            }
            NodeKind::ExprUnaryOperator { kind, value } => {
                PolishNodeTree::Branch(
                    match kind {
                        OperatorKind::Ref => "ref",
                        OperatorKind::Deref => "deref",
                        OperatorKind::Not => "not",
                        OperatorKind::Inc => "inc",
                        OperatorKind::Dec => "dec",
                        OperatorKind::Negate => "negate",
                        _ => return None, // error
                    }
                    .to_string(),
                    vec![PolishNodeTree::from_node(*value.clone()).unwrap()],
                )
            }
            NodeKind::ExprBinaryOperator { kind, left, right } => {
                PolishNodeTree::Branch(
                    match kind {
                        OperatorKind::Dot => ".",
                        OperatorKind::As => "as",
                        OperatorKind::Assign => "=",
                        OperatorKind::And => "and",
                        OperatorKind::Or => "or",
                        OperatorKind::Eq => "==",
                        OperatorKind::Neq => "!=",
                        OperatorKind::Gt => ">",
                        OperatorKind::GtEq => ">=",
                        OperatorKind::Lt => "<",
                        OperatorKind::LtEq => "<=",
                        OperatorKind::Add => "+",
                        OperatorKind::Sub => "-",
                        OperatorKind::Mul => "*",
                        OperatorKind::Div => "/",
                        OperatorKind::Mod => "%",
                        _ => return None, // error
                    }
                    .to_string(),
                    vec![
                        PolishNodeTree::from_node(*left.clone()).unwrap(),
                        PolishNodeTree::from_node(*right.clone()).unwrap(),
                    ],
                )
            }
            NodeKind::ExprInvoke { left, params } => PolishNodeTree::Branch(
                format!(
                    "invoke={}",
                    PolishNodeTree::from_node(*left.clone()).unwrap()
                )
                .to_string(),
                PolishNodeTree::from_node_vec(params).unwrap(),
            ),
            NodeKind::ExprList(nodes) => PolishNodeTree::Branch(
                "list".to_string(),
                PolishNodeTree::from_node_vec(nodes).unwrap(),
            ),
            NodeKind::ExprVar {
                name,
                typ: _typ,
                value,
            } => PolishNodeTree::Branch(
                "var".to_string(),
                vec![
                    PolishNodeTree::Leaf(name),
                    PolishNodeTree::from_node(*value.clone()).unwrap(),
                ],
            ),
            NodeKind::ExprLet {
                name,
                typ: _typ,
                value,
            } => PolishNodeTree::Branch(
                "let".to_string(),
                vec![
                    PolishNodeTree::Leaf(name),
                    PolishNodeTree::from_node(*value.clone()).unwrap(),
                ],
            ),
            _ => return None,
        })
    }
}

impl fmt::Display for PolishNodeTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolishNodeTree::Leaf(value) => write!(f, "{VALUE}{}{RESET}", value),
            PolishNodeTree::Branch(text, children) => {
                _ = write!(f, "({NAME}{}{RESET}", text);
                for s in children {
                    _ = write!(f, " {}", s);
                }
                write!(f, ")")
            }
        }
    }
}

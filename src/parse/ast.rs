use std::{collections::HashMap, fmt, path::PathBuf};

use crate::hashtags;

use super::operator::OperatorKind;

#[derive(Debug, Clone)]
pub enum Node {
    Program(Vec<Node>),
    Raw(String),
    Type {
        pointers: u8,
        name: String,
        arrays: Vec<(Option<usize>, Option<String>)>,
        funptr_args: Option<Vec<Node>>,
        funptr_rets: Option<Box<Node>>,
    },
    // Top level statements
    TopUse(PathBuf),
    TopFun {
        tags: Vec<hashtags::FunTags>,
        id: String,
        params: HashMap<String, Node>,
        rets: Box<Node>,
        expr: Box<Node>,
    },
    TopRec {
        tags: Vec<hashtags::RecTags>,
        id: String,
        params: HashMap<String, Node>,
    },
    TopDef {
        id: String,
        typ: Box<Node>,
    },
    TopMac {
        tags: Vec<hashtags::MacTags>,
        id: String,
        params: Vec<String>,
        returns: Option<Box<Node>>,
        expands_to: String,
    },
    TopTag {
        tags: Vec<hashtags::TagTags>,
        id: String,
        entries: Vec<String>,
    },
    TopTagRec {
        tags: Vec<hashtags::TagRecTags>,
        id: String,
        entries: Vec<(String, HashMap<String, Node>)>,
    },
    // Statements
    StatRet(Option<Box<Node>>),
    StatIf {
        cond: Box<Node>,
        expr: Box<Node>,
        else_: Option<Box<Node>>,
    },
    StatSwitch {
        switch: Box<Node>,
        cases: HashMap<Option<Box<Node>>, Box<Node>>,
    },
    StatExpr(Box<Node>),
    // Expressions
    ExprGroup(Box<Node>),
    ExprNumber(String),
    ExprString(String),
    ExprChar(char),
    ExprTrue,
    ExprFalse,
    ExprIdentifier(String),
    ExprBlock(Vec<Node>),
    ExprNew {
        id: String,
        params: Vec<Node>,
    },
    ExprUnaryOperator {
        kind: OperatorKind,
        value: Box<Node>,
    },
    ExprBinaryOperator {
        kind: OperatorKind,
        left: Box<Node>,
        right: Box<Node>,
    },
    ExprInvoke {
        left: Box<Node>,
        params: Vec<Node>,
    },
    ExprMacInvoke {
        left: Box<Node>,
        params: Vec<Node>,
    },
    ExprList(Vec<Node>),
    ExprVar {
        name: String,
        value: Box<Node>,
    },
    ExprLet {
        name: String,
        value: Box<Node>,
    },
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Node {
    pub fn get_void_type() -> Self {
        Node::Type {
            pointers: 0,
            name: "void".to_string(),
            arrays: vec![],
            funptr_args: None,
            funptr_rets: None,
        }
    }

    pub fn join(kind: OperatorKind, left: Node, right: Node) -> Self {
        Node::ExprBinaryOperator {
            kind,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

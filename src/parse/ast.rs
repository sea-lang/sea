use std::{fmt, path::PathBuf};

use crate::hashtags;

use super::operator::OperatorKind;

#[derive(Debug, Clone)]
pub struct Node {
    pub line: usize,
    pub column: usize,
    pub node: NodeKind,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
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
    TopUse(PathBuf, Option<Vec<String>>),
    TopFun {
        tags: Vec<hashtags::FunTags>,
        id: String,
        params: Vec<(String, Node)>,
        rets: Box<Node>,
        expr: Box<Node>,
    },
    TopRec {
        tags: Vec<hashtags::RecTags>,
        id: String,
        fields: Vec<(String, Node)>,
    },
    TopDef {
        tags: Vec<hashtags::DefTags>,
        id: String,
        typ: Box<Node>,
    },
    TopMac {
        tags: Vec<hashtags::MacTags>,
        id: String,
        params: Vec<String>,
        rets: Option<Box<Node>>,
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
        entries: Vec<(String, Vec<(String, Node)>)>,
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
        // A list of: (case expression [if None then this is the `else` case], is fall case, case code block)
        cases: Vec<(Option<Box<Node>>, bool, Box<Node>)>,
    },
    StatForCStyle {
        def: Box<Node>,
        cond: Box<Node>,
        inc: Box<Node>,
        expr: Box<Node>,
    },
    StatForSingleExpr {
        cond: Box<Node>,
        expr: Box<Node>,
    },
    StatForRange {
        var: Option<String>,
        from: Box<Node>,
        to: Box<Node>,
        expr: Box<Node>,
    },
    // StatForIn {},
    StatExpr(Box<Node>),
    // Expressions
    ExprGroup(Box<Node>),
    ExprNumber(String),
    ExprString(String),
    ExprCString(String),
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
        name: String,
        params: Vec<Node>,
    },
    ExprList(Vec<Node>),
    ExprVar {
        name: String,
        typ: Option<Box<Node>>,
        value: Box<Node>,
    },
    ExprLet {
        name: String,
        typ: Option<Box<Node>>,
        value: Box<Node>,
    },
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Node {
    pub fn get_void_type(line: usize, column: usize) -> Node {
        Node {
            line,
            column,
            node: NodeKind::Type {
                pointers: 0,
                name: "void".to_string(),
                arrays: vec![],
                funptr_args: None,
                funptr_rets: None,
            },
        }
    }

    pub fn join(kind: OperatorKind, left: Node, right: Node) -> Self {
        Node {
            line: left.line,
            column: left.column,
            node: NodeKind::ExprBinaryOperator {
                kind,
                left: Box::new(left),
                right: Box::new(right),
            },
        }
    }
}

use std::{collections::HashMap, fmt, path::PathBuf};

use crate::hashtags;

#[derive(Debug, Clone)]
pub struct BinOp {
    left: Box<Node>,
    right: Box<Node>,
}

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
    ExprBlock(Vec<Node>),
    ExprNew {
        id: String,
        params: Vec<Node>,
    },
    ExprDot(BinOp),
    ExprRef(Box<Node>),
    ExprDeref(Box<Node>),
    ExprAssign(BinOp),
    ExprNot(Box<Node>),
    ExprAnd(BinOp),
    ExprOr(BinOp),
    ExprEq(BinOp),
    ExprNeq(BinOp),
    ExprGt(BinOp),
    ExprGtEq(BinOp),
    ExprLt(BinOp),
    ExprLtEq(BinOp),
    ExprAdd(BinOp),
    ExprSub(BinOp),
    ExprMul(BinOp),
    ExprDiv(BinOp),
    ExprInc(BinOp),
    ExprDec(BinOp),
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
    ExprAs(BinOp),
    ExprId(String),
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

    pub fn pretty_print(&self, indent: usize) {
        print!("{}", "\t".repeat(indent));
        match self {
            Node::Program(nodes) => {
                println!("program:");
                for node in nodes {
                    node.pretty_print(indent + 1);
                }
            }
            Node::Raw(code) => println!("raw code: `{}`", code),
            Node::Type {
                pointers: _,
                name: _,
                arrays: _,
                funptr_args: _,
                funptr_rets: _,
            } => println!("type: {}", self),
            Node::TopUse(path_buf) => println!("use: {:?}", path_buf),
            Node::TopFun {
                tags,
                id,
                params,
                rets,
                expr,
            } => {
                println!("fun: #{:?} {} {:?}: {}", tags, id, "params", "rets");
                expr.pretty_print(indent + 1);
            }
            Node::TopRec { tags, id, params } => todo!(),
            Node::TopDef { id, typ } => todo!(),
            Node::TopMac {
                tags,
                id,
                params,
                returns,
                expands_to,
            } => todo!(),
            Node::TopTag { tags, id, entries } => todo!(),
            Node::TopTagRec { tags, id, entries } => todo!(),
            Node::StatRet(node) => {
                println!("ret:");
                node.as_ref().unwrap().pretty_print(indent + 1);
            }
            Node::StatIf { cond, expr, else_ } => todo!(),
            Node::StatSwitch { switch, cases } => todo!(),
            Node::StatExpr(node) => {
                println!("expr:");
                node.pretty_print(indent + 1);
            }
            Node::ExprGroup(node) => {
                println!("group:");
                node.pretty_print(indent + 1);
            }
            Node::ExprNumber(value) => println!("number: '{}'", value),
            Node::ExprString(value) => println!("string: '{}'", value),
            Node::ExprChar(value) => println!("char: '{}'", value),
            Node::ExprTrue => println!("true"),
            Node::ExprFalse => println!("false"),
            Node::ExprBlock(nodes) => {
                println!("block:");
                for node in nodes {
                    node.pretty_print(indent + 1);
                }
            }
            Node::ExprNew { id, params } => todo!(),
            Node::ExprDot(bin_op) => todo!(),
            Node::ExprRef(node) => todo!(),
            Node::ExprDeref(node) => todo!(),
            Node::ExprAssign(bin_op) => todo!(),
            Node::ExprNot(node) => todo!(),
            Node::ExprAnd(bin_op) => todo!(),
            Node::ExprOr(bin_op) => todo!(),
            Node::ExprEq(bin_op) => todo!(),
            Node::ExprNeq(bin_op) => todo!(),
            Node::ExprGt(bin_op) => todo!(),
            Node::ExprGtEq(bin_op) => todo!(),
            Node::ExprLt(bin_op) => todo!(),
            Node::ExprLtEq(bin_op) => todo!(),
            Node::ExprAdd(bin_op) => todo!(),
            Node::ExprSub(bin_op) => todo!(),
            Node::ExprMul(bin_op) => todo!(),
            Node::ExprDiv(bin_op) => todo!(),
            Node::ExprInc(bin_op) => todo!(),
            Node::ExprDec(bin_op) => todo!(),
            Node::ExprInvoke { left, params } => todo!(),
            Node::ExprMacInvoke { left, params } => todo!(),
            Node::ExprList(nodes) => todo!(),
            Node::ExprVar { name, value } => todo!(),
            Node::ExprLet { name, value } => todo!(),
            Node::ExprAs(bin_op) => todo!(),
            Node::ExprId(value) => println!("id: '{}'", value),
        }
    }
}

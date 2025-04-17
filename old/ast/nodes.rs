use std::{collections::HashMap, path::PathBuf};

use crate::{hashtags::FunTags, type_::SeaType};

pub trait ASTNode {}

pub struct NodeProgram {
    statements: Vec<Box<dyn ASTNode>>,
}

impl ASTNode for NodeProgram {}

pub struct NodeType {
    pub pointers: u8,
    pub name: String,
    pub arrays: Vec<u8>,
    pub funptr_args: Vec<Box<NodeType>>,
    pub funptr_rets: Option<Box<NodeType>>,
}

impl ASTNode for NodeType {}

pub enum TopLevelStatement {
    Use{ path: PathBuf },
    Fun {
        name: String,
        hashtags: Vec<FunTags>,
        params: HashMap<String, SeaType>,
        rets: SeaType,
        expr: Expr,
    },
}

impl ASTNode for TopLevelStatement {}

pub enum Expr {
    ExprBlock(Vec<Box<Expr>>),
}

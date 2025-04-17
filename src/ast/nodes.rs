use std::collections::HashMap;

use crate::hashtags::FunTags;

pub trait ASTNode {}

pub struct NodeProgram {
    statements: Vec<dyn ASTNode>,
}

impl ASTNode for NodeProgram {}

pub enum TopLevelStatement {
    Use(PathBuf),
    Fun {
        name: String,
        hashtags: Vec<FunTags>,
        params: HashMap<String, SeaType>,
        rets: SeaType,
        expr: Expr,
    },
}

// pub enum ASTNode {
//     Program { stats: Vec<ASTNode> },
//     TopLevelStat { stat: ASTNode },
//     StatUse { path: PathBuf },
// }

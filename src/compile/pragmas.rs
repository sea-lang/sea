use strum::EnumString;

use crate::parse::ast::{Node, NodeKind};

use super::error::CompilerError;

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
pub enum Pragma {
    AddCCFlag(String),
    AddLibrary(String),
    AddIncludeDir(String),
}

impl Pragma {
    fn get_single_string_pragma(params: &Vec<Node>) -> Result<String, CompilerError> {
        if params.len() != 1 {
            return Err(CompilerError::NotEnoughOrTooManyPragmaArguments(
                1,
                params.len(),
            ));
        }

        match &params.get(0).unwrap().node {
            NodeKind::ExprString(it) => Ok(it.clone()),
            _ => Err(CompilerError::InvalidPragmaArguments(
                "String".to_string(),
                0,
            )),
        }
    }

    pub fn from(id: &String, params: &Vec<Node>) -> Result<Pragma, CompilerError> {
        Ok(match id.as_str() {
            "add_cc_flag" => Pragma::AddCCFlag(Self::get_single_string_pragma(params)?),
            "add_library" => Pragma::AddLibrary(Self::get_single_string_pragma(params)?),
            "add_include_dir" => Pragma::AddIncludeDir(Self::get_single_string_pragma(params)?),
            _ => return Err(CompilerError::NoSuchPragma(id.clone())),
        })
    }

    pub fn from_node(node: &Node) -> Result<Pragma, CompilerError> {
        match &node.node {
            NodeKind::TopPragma { id, params } => Pragma::from(id, params),
            _ => unreachable!(),
        }
    }
}

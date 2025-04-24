use std::sync::LazyLock;

use crate::parse::ast::{Node, NodeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeaType {
    pub pointers: u8,
    pub name: String,
    pub arrays: Vec<(Option<usize>, Option<String>)>,
    pub funptr_args: Option<Vec<SeaType>>,
    pub funptr_rets: Option<Box<SeaType>>,
}

impl SeaType {
    pub fn from_node(node: Node) -> Option<Self> {
        match node.node {
            NodeKind::Type {
                pointers,
                name,
                arrays,
                funptr_args,
                funptr_rets,
            } => {
                let seatype_funptr_args = if funptr_args.is_some() {
                    let mut args: Vec<SeaType> = vec![];
                    for arg in funptr_args.unwrap() {
                        args.push(SeaType::from_node(arg).unwrap());
                    }
                    Some(args)
                } else {
                    None
                };

                let seatype_funptr_rets =
                    funptr_rets.map(|it| Box::new(SeaType::from_node(*it).unwrap()));

                Some(SeaType {
                    pointers,
                    name,
                    arrays,
                    funptr_args: seatype_funptr_args,
                    funptr_rets: seatype_funptr_rets,
                })
            }
            _ => None,
        }
    }

    pub fn named_type(name: &str) -> Self {
        SeaType {
            pointers: 0,
            name: name.to_string(),
            arrays: vec![],
            funptr_args: None,
            funptr_rets: None,
        }
    }

    pub fn pointer(&self) -> Self {
        SeaType {
            pointers: self.pointers + 1,
            ..self.clone()
        }
    }

    pub fn unpointer(&self) -> Self {
        SeaType {
            pointers: self.pointers - 1,
            ..self.clone()
        }
    }

    pub fn array(&self) -> Self {
        let mut arrays = self.arrays.clone();
        arrays.push((None, None));
        SeaType {
            arrays,
            ..self.clone()
        }
    }

    pub fn array_of_size(&self, size: usize) -> Self {
        let mut arrays = self.arrays.clone();
        arrays.push((Some(size), None));
        SeaType {
            arrays,
            ..self.clone()
        }
    }

    pub const BOOL: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("bool"));

    pub const I8: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("i8"));
    pub const I16: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("i16"));
    pub const I32: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("i32"));
    pub const I64: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("i64"));

    pub const U8: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("u8"));
    pub const U16: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("u16"));
    pub const U32: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("u32"));
    pub const U64: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("u64"));

    pub const F32: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("f32"));
    pub const F64: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("f64"));

    pub const CHAR: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("char"));
    pub const C_STRING: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("char").pointer());
    pub const STRING: LazyLock<SeaType> = LazyLock::new(|| SeaType::named_type("String"));
}

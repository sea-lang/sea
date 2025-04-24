use crate::parse::ast::{Node, NodeKind};

pub struct SeaType {
    pub pointers: u8,
    pub name: String,
    pub arrays: Vec<(Option<usize>, Option<String>)>,
    pub funptr_args: Option<Vec<SeaType>>,
    pub funptr_rets: Option<Box<SeaType>>,
}

impl SeaType {
    pub const INTEGER: SeaType = SeaType {
        pointers: 0,
        name: "i32".to_string(),
        arrays: todo!(),
        funptr_args: todo!(),
        funptr_rets: todo!(),
    };

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
}

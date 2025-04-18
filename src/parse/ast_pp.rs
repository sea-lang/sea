// Pretty printer for AST nodes

use super::ast::Node;

const RESET: &'static str = "\x1b[0m";
const TYPE: &'static str = "\x1b[33m";
const TOP_LEVEL_STAT: &'static str = "\x1b[34m";
const STAT: &'static str = "\x1b[32m";
const EXPR: &'static str = "\x1b[31m";

impl Node {
    pub fn pretty_print(&self) {
        println!(
            "\
\x1b[1mnodes are colour-coded using the following key:\x1b[0m
  types: {TYPE}yellow{RESET}
  top level statements: {TOP_LEVEL_STAT}blue{RESET}
  statements: {STAT}green{RESET}
  expressions: {EXPR}red{RESET}
"
        );

        self.pretty_print_inner(0, false);
    }

    fn pretty_print_inner(&self, indent: usize, indent_first: bool) {
        let spacing = "  ".repeat(indent);
        if indent_first {
            print!("{}", spacing);
        }

        match self {
            Node::Program(nodes) => {
                println!("\x1b[1mprogram:\x1b[0m");
                for node in nodes {
                    node.pretty_print_inner(indent + 1, true);
                }
                print!("\x1b[0m");
            }
            Node::Raw(code) => println!("raw code: `{code}`"),
            Node::Type {
                pointers,
                name,
                arrays,
                funptr_args,
                funptr_rets,
            } => {
                print!("{TYPE}");
                // function pointer types are complex, so we'll write them over multiple lines
                if funptr_rets.is_some() {
                    println!("type:");
                    // name
                    println!("{spacing}  name: {name}");
                    // pointers
                    println!(
                        "{spacing}  pointers: {} ({})",
                        "^".repeat(*pointers as usize),
                        pointers
                    );
                    // arrays
                    print!("{spacing}  arrays: ");
                    for (array_size_opt, array_id_opt) in arrays {
                        if array_size_opt.is_none() && array_id_opt.is_none() {
                            print!("[]")
                        } else if array_size_opt.is_some() {
                            print!("[{}]", array_size_opt.unwrap())
                        } else if array_id_opt.is_some() {
                            print!("[{}]", array_id_opt.as_ref().unwrap())
                        }
                    }
                    println!(" ({})", arrays.iter().count());
                    // parameters
                    println!("{spacing}  args: (");
                    for arg in funptr_args.as_ref().unwrap() {
                        arg.pretty_print_inner(indent + 2, true);
                    }
                    println!("{TYPE}{}  ): ", spacing);
                    // returns
                    print!("{spacing}  rets: ");
                    funptr_rets
                        .as_ref()
                        .unwrap()
                        .pretty_print_inner(indent, false);
                } else {
                    print!("type: {}{}", "^".repeat(*pointers as usize), name);
                    for (array_size_opt, array_id_opt) in arrays {
                        if array_size_opt.is_none() && array_id_opt.is_none() {
                            print!("[]")
                        } else if array_size_opt.is_some() {
                            print!("[{}]", array_size_opt.unwrap())
                        } else if array_id_opt.is_some() {
                            print!("[{}]", array_id_opt.as_ref().unwrap())
                        }
                    }
                    println!("");
                }
            }
            Node::TopUse(path_buf) => println!("{TOP_LEVEL_STAT}use: {path_buf:?}{RESET}"),
            Node::TopFun {
                tags,
                id,
                params,
                rets,
                expr,
            } => {
                println!("{TOP_LEVEL_STAT}fun: {id}");

                if tags.iter().count() > 0 {
                    println!("{spacing}  tags: #{tags:?}");
                }

                println!("{spacing}  params:");
                for (param_name, param_node) in params {
                    print!("{TOP_LEVEL_STAT}{spacing}    {param_name} = ");
                    param_node.pretty_print_inner(indent + 2, false);
                }

                print!("{TOP_LEVEL_STAT}{spacing}  rets = ");
                rets.pretty_print_inner(indent, false);

                println!("{TOP_LEVEL_STAT}{spacing}  code:");
                expr.pretty_print_inner(indent + 2, true);
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
                println!("{STAT}ret:");
                node.as_ref().unwrap().pretty_print_inner(indent + 1, true);
            }
            Node::StatIf { cond, expr, else_ } => todo!(),
            Node::StatSwitch { switch, cases } => todo!(),
            Node::StatExpr(node) => {
                println!("{EXPR}expr:");
                node.pretty_print_inner(indent + 1, true);
            }
            Node::ExprGroup(node) => {
                println!("{EXPR}group:");
                node.pretty_print_inner(indent + 1, true);
            }
            Node::ExprNumber(value) => println!("{EXPR}number: '{}'", value),
            Node::ExprString(value) => println!("{EXPR}string: '{}'", value),
            Node::ExprChar(value) => println!("{EXPR}char: '{}'", value),
            Node::ExprTrue => println!("{EXPR}true"),
            Node::ExprFalse => println!("{EXPR}false"),
            Node::ExprIdentifier(value) => println!("{EXPR}id: '{}'", value),
            Node::ExprBlock(nodes) => {
                println!("{EXPR}block:");
                for node in nodes {
                    node.pretty_print_inner(indent + 1, true);
                }
            }
            Node::ExprNew { id, params } => {
                println!("{EXPR}new: {}, params:", id);
                for param in params {
                    param.pretty_print_inner(indent + 1, true);
                }
            }
            Node::ExprUnaryOperator { kind, value } => {
                println!("{EXPR}unary op: {:?}", kind);
                print!("{spacing}  value: ");
                value.pretty_print_inner(indent + 1, false);
            }
            Node::ExprBinaryOperator { kind, left, right } => {
                println!("{EXPR}binary op: {:?}", kind);
                print!("{spacing}  left: ");
                left.pretty_print_inner(indent + 1, false);
                print!("{spacing}  {EXPR}right: ");
                right.pretty_print_inner(indent + 1, false);
            }
            Node::ExprInvoke { left, params } => todo!(),
            Node::ExprMacInvoke { left, params } => todo!(),
            Node::ExprList(nodes) => todo!(),
            Node::ExprVar { name, value } => todo!(),
            Node::ExprLet { name, value } => todo!(),
        }

        print!("{RESET}");
    }
}

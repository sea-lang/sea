// Pretty printer for AST nodes

use super::ast::Node;

const RESET: &'static str = "\x1b[0m";
const TYPE: &'static str = "\x1b[33m";
const TOP_LEVEL_STAT: &'static str = "\x1b[34m";
const STAT: &'static str = "\x1b[32m";
const EXPR: &'static str = "\x1b[31m";
const TEXT: &'static str = "\x1b[36m";

const HELP: &'static str = "\
\x1b[1mnodes are colour-coded using the following key:\x1b[0m
  types: {TYPE}yellow{RESET}
  top level statements: {TOP_LEVEL_STAT}blue{RESET}
  statements: {STAT}green{RESET}
  expressions: {EXPR}red{RESET}
  values: {TEXT}cyan{RESET}
";

impl Node {
    pub fn print_help() {
        println!("{}", HELP);
    }

    pub fn pretty_print(&self) {
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
            Node::Raw(code) => println!("raw code: '{TEXT}{code}{RESET}'"),
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
                    println!("{spacing}  name: {TEXT}{name}{TYPE}");
                    // pointers
                    println!(
                        "{spacing}  pointers: {TEXT}{}{TYPE} ({TEXT}{}{TYPE})",
                        "^".repeat(*pointers as usize),
                        pointers
                    );
                    // arrays
                    print!("{spacing}  arrays: {TEXT}");
                    for (array_size_opt, array_id_opt) in arrays {
                        if array_size_opt.is_none() && array_id_opt.is_none() {
                            print!("[]")
                        } else if array_size_opt.is_some() {
                            print!("[{}]", array_size_opt.unwrap())
                        } else if array_id_opt.is_some() {
                            print!("[{}]", array_id_opt.as_ref().unwrap())
                        }
                    }
                    println!("{TYPE} ({TEXT}{}{TYPE})", arrays.iter().count());
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
                    print!("type: {TEXT}{}{}", "^".repeat(*pointers as usize), name);
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
            Node::TopUse(path_buf) => println!("{TOP_LEVEL_STAT}use: {TEXT}{path_buf:?}"),
            Node::TopFun {
                tags,
                id,
                params,
                rets,
                expr,
            } => {
                println!("{TOP_LEVEL_STAT}fun '{TEXT}{id}{TOP_LEVEL_STAT}':");

                if tags.iter().count() > 0 {
                    println!("{spacing}  tags: {TEXT}#{tags:?}{TOP_LEVEL_STAT}");
                }

                println!("{spacing}  params:");
                for (param_name, param_node) in params {
                    print!("{TOP_LEVEL_STAT}{spacing}    '{TEXT}{param_name}{TOP_LEVEL_STAT}' = ");
                    param_node.pretty_print_inner(indent + 2, false);
                }

                print!("{TOP_LEVEL_STAT}{spacing}  rets = ");
                rets.pretty_print_inner(indent, false);

                println!("{TOP_LEVEL_STAT}{spacing}  code:");
                expr.pretty_print_inner(indent + 2, true);
            }
            Node::TopRec { tags, id, fields } => {
                println!("{TOP_LEVEL_STAT}rec '{TEXT}{id}{TOP_LEVEL_STAT}'");

                if tags.iter().count() > 0 {
                    println!("{spacing}  tags: {TEXT}#{tags:?}{TOP_LEVEL_STAT}");
                }

                println!("{spacing}  fields:");
                for (field_name, field_node) in fields {
                    print!("{TOP_LEVEL_STAT}{spacing}    {TEXT}{field_name}{TOP_LEVEL_STAT} = ");
                    field_node.pretty_print_inner(indent + 2, false);
                }
            }
            Node::TopDef { tags, id, typ } => {
                print!("{TOP_LEVEL_STAT}def '{TEXT}{id}{TOP_LEVEL_STAT}' = ");
                typ.pretty_print_inner(indent + 1, false);

                if tags.iter().count() > 0 {
                    println!("{spacing}  tags: {TEXT}#{tags:?}{TOP_LEVEL_STAT}");
                }
            }
            Node::TopMac {
                tags,
                id,
                params,
                rets,
                expands_to,
            } => {
                println!("{TOP_LEVEL_STAT}mac '{TEXT}{id}{TOP_LEVEL_STAT}':");

                if tags.iter().count() > 0 {
                    println!("{spacing}  tags: {TEXT}#{tags:?}{TOP_LEVEL_STAT}");
                }

                println!("{spacing}  params:");
                for param_name in params {
                    println!("{TOP_LEVEL_STAT}{spacing}    - '{TEXT}{param_name}{TOP_LEVEL_STAT}'");
                }

                if let Some(rets) = rets {
                    print!("{TOP_LEVEL_STAT}{spacing}  rets = ");
                    rets.pretty_print_inner(indent, false);
                }

                println!(
                    "{TOP_LEVEL_STAT}{spacing}  expansion = '{TEXT}{expands_to}{TOP_LEVEL_STAT}'"
                );
            }
            Node::TopTag { tags, id, entries } => {
                println!("{TOP_LEVEL_STAT}tag '{TEXT}{id}{TOP_LEVEL_STAT}':");

                if tags.iter().count() > 0 {
                    println!("{spacing}  tags: {TEXT}#{tags:?}{TOP_LEVEL_STAT}");
                }

                println!("{spacing}  entries:");
                for entry in entries {
                    println!("{spacing}    - '{TEXT}{entry}{TOP_LEVEL_STAT}'");
                }
            }
            Node::TopTagRec { tags, id, entries } => {
                println!("{TOP_LEVEL_STAT}tag rec '{TEXT}{id}{TOP_LEVEL_STAT}':");

                if tags.iter().count() > 0 {
                    println!("{spacing}  tags: {TEXT}#{tags:?}{TOP_LEVEL_STAT}");
                }

                println!("{spacing}  entries:");
                for (entry_id, entry_entries) in entries {
                    if entry_entries.len() > 0 {
                        println!("{spacing}    - '{TEXT}{entry_id}{TOP_LEVEL_STAT}'(");
                        for (entry_entry_name, entry_entry_typ) in entry_entries {
                            print!("{TOP_LEVEL_STAT}{spacing}      '{TEXT}{entry_entry_name}{TOP_LEVEL_STAT}' = ");
                            entry_entry_typ.pretty_print_inner(indent + 4, false);
                        }
                        println!("{TOP_LEVEL_STAT}{spacing}    )")
                    } else {
                        println!("{spacing}    - '{TEXT}{entry_id}{TOP_LEVEL_STAT}'()");
                    }
                }
            }
            Node::StatRet(node) => {
                println!("{STAT}ret:");
                node.as_ref().unwrap().pretty_print_inner(indent + 1, true);
            }
            Node::StatIf { cond, expr, else_ } => {
                println!("{STAT}if:");
                println!("{spacing}  cond:");
                cond.pretty_print_inner(indent + 2, true);
                println!("{STAT}{spacing}  expr:");
                expr.pretty_print_inner(indent + 2, true);
                if let Some(it) = else_ {
                    println!("{STAT}{spacing}  else:");
                    it.pretty_print_inner(indent + 2, true);
                }
            }
            Node::StatSwitch { switch, cases } => {
                println!("{STAT}switch:");
                println!("{spacing}  expr:");
                switch.pretty_print_inner(indent + 2, true);
                println!("{STAT}{spacing}  cases:");
                for (case, is_fall_case, block) in cases {
                    match case {
                        Some(case) => {
                            if *is_fall_case {
                                println!("{STAT}{spacing}    fall case:");
                            } else {
                                println!("{STAT}{spacing}    case:");
                            }
                            case.pretty_print_inner(indent + 3, true);
                        }
                        None => println!("{STAT}{spacing}    else:"),
                    }
                    println!("{STAT}{spacing}      code:");
                    block.pretty_print_inner(indent + 4, true);
                }
            }
            Node::StatForCStyle {
                def,
                cond,
                inc,
                expr,
            } => {
                println!("{STAT}for (c style):");
                println!("{spacing}  def: ");
                def.pretty_print_inner(indent + 2, true);
                println!("{STAT}{spacing}  cond: ");
                cond.pretty_print_inner(indent + 2, true);
                println!("{STAT}{spacing}  inc: ");
                inc.pretty_print_inner(indent + 2, true);
                println!("{STAT}{spacing}  expr:");
                expr.pretty_print_inner(indent + 2, true);
            }
            Node::StatForSingleExpr { cond, expr } => {
                println!("{STAT}for (single expr):");
                println!("{spacing}  cond: ");
                cond.pretty_print_inner(indent + 2, true);
                println!("{STAT}{spacing}  expr:");
                expr.pretty_print_inner(indent + 2, true);
            }
            Node::StatForRange {
                var,
                from,
                to,
                expr,
            } => {
                println!("{STAT}for (range):");
                if let Some(it) = var {
                    println!("{spacing}  var: {it}");
                }
                println!("{spacing}  from: ");
                from.pretty_print_inner(indent + 2, true);
                println!("{STAT}{spacing}  to: ");
                to.pretty_print_inner(indent + 2, true);
                println!("{STAT}{spacing}  expr:");
                expr.pretty_print_inner(indent + 2, true);
            }
            Node::StatExpr(node) => {
                println!("{EXPR}expr:");
                node.pretty_print_inner(indent + 1, true);
            }
            Node::ExprGroup(node) => {
                println!("{EXPR}group:");
                node.pretty_print_inner(indent + 1, true);
            }
            Node::ExprNumber(value) => println!("{EXPR}number: '{TEXT}{value}{EXPR}'"),
            Node::ExprString(value) => println!("{EXPR}string: '{TEXT}{value}{EXPR}'"),
            Node::ExprCString(value) => println!("{EXPR}cstring: c'{TEXT}{value}{EXPR}'"),
            Node::ExprChar(value) => println!("{EXPR}char: '{TEXT}{value}{EXPR}'"),
            Node::ExprTrue => println!("{EXPR}true"),
            Node::ExprFalse => println!("{EXPR}false"),
            Node::ExprIdentifier(value) => println!("{EXPR}id: '{TEXT}{value}{EXPR}'"),
            Node::ExprBlock(nodes) => {
                println!("{EXPR}block:");
                for node in nodes {
                    node.pretty_print_inner(indent + 1, true);
                }
            }
            Node::ExprNew { id, params } => {
                println!("{EXPR}new: '{TEXT}{id}{EXPR}', params:");
                for param in params {
                    param.pretty_print_inner(indent + 1, true);
                }
            }
            Node::ExprUnaryOperator { kind, value } => {
                println!("{EXPR}unary op: '{TEXT}{kind:?}{EXPR}'");
                print!("{spacing}  value: ");
                value.pretty_print_inner(indent + 1, false);
            }
            Node::ExprBinaryOperator { kind, left, right } => {
                println!("{EXPR}binary op: '{TEXT}{kind:?}{EXPR}'");
                print!("{spacing}  left: ");
                left.pretty_print_inner(indent + 1, false);
                print!("{EXPR}{spacing}  right: ");
                right.pretty_print_inner(indent + 1, false);
            }
            Node::ExprInvoke { left, params } => {
                println!("{EXPR}invoke:");
                left.pretty_print_inner(indent + 1, true);
                println!("{EXPR}{spacing}  params:");
                for param in params {
                    print!("{EXPR}{spacing}    - ");
                    param.pretty_print_inner(indent + 2, false);
                }
            }
            Node::ExprMacInvoke { name, params } => {
                println!("mac invoke: '{TEXT}{name}{RESET}'");
                println!("{spacing}  params:");
                for param in params {
                    print!("{RESET}{spacing}    - ");
                    param.pretty_print_inner(indent + 2, false);
                }
            }
            Node::ExprList(nodes) => {
                println!("{EXPR}list:");
                for node in nodes {
                    print!("{EXPR}{spacing}  - ");
                    node.pretty_print_inner(indent + 1, false);
                }
            }
            Node::ExprVar { name, typ, value } => match typ {
                Some(it) => {
                    println!("{EXPR}var '{TEXT}{name}{EXPR}':");
                    print!("{EXPR}{spacing}  type: ");
                    it.pretty_print_inner(indent + 1, false);
                    print!("{EXPR}{spacing}  value = ");
                    value.pretty_print_inner(indent + 1, false);
                }
                None => {
                    println!("{EXPR}var '{TEXT}{name}{EXPR}' =");
                    value.pretty_print_inner(indent + 1, true);
                }
            },
            Node::ExprLet { name, typ, value } => match typ {
                Some(it) => {
                    println!("{EXPR}let '{TEXT}{name}{EXPR}':");
                    print!("{EXPR}{spacing}  type: ");
                    it.pretty_print_inner(indent + 1, false);
                    print!("{EXPR}{spacing}  value = ");
                    value.pretty_print_inner(indent + 1, false);
                }
                None => {
                    println!("{EXPR}let '{TEXT}{name}{EXPR}' =");
                    value.pretty_print_inner(indent + 1, true);
                }
            },
        }

        print!("{RESET}");
    }
}

use core::{fmt, panic};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Write},
    path::PathBuf,
};

use crate::{
    backend::backend::Backend,
    compile::{compiler::Compiler, symbol::Symbol},
    hashtags::{FunTags, RecTags},
    parse::{ast::Node, operator::OperatorKind},
};

pub struct CBackend {
    pub compiler: Compiler,
}

impl CBackend {
    pub fn w(&mut self, fmt: fmt::Arguments<'_>) {
        self.compiler
            .output_file
            .write_fmt(fmt)
            .expect("failed to write to output file");
    }

    pub fn ws(&mut self, s: &str) {
        self.compiler
            .output_file
            .write(s.as_bytes())
            .expect("failed to write to output file");
    }

    pub fn comma_separated(&mut self, nodes: Vec<Node>) {
        if nodes.len() == 0 {
            return;
        }

        let end = nodes.len() - 1;
        let mut index = 0;
        for node in nodes {
            self.write(node);
            if index != end {
                self.ws(", ");
            }
            index += 1
        }
    }

    pub fn program(&mut self, program: Vec<Node>) {
        let file_path = self.compiler.output_path.clone();
        self.w(format_args!(
            "#pragma region \"file: {}\"\n",
            file_path.to_str().unwrap()
        ));

        for node in program {
            self.write(node);
        }

        self.w(format_args!(
            "#pragma region \"end file: {}\"\n",
            file_path.to_str().unwrap()
        ));
    }

    pub fn raw(&mut self, text: String) {
        self.w(format_args!("{}\n", text))
    }

    fn get_type_array_str(arrays: Vec<(Option<usize>, Option<String>)>) -> String {
        arrays
            .iter()
            .map(|it| {
                if it.0.is_some() {
                    format!("[{}]", it.0.unwrap())
                } else if it.1.is_some() {
                    format!("[{}]", it.1.clone().unwrap())
                } else {
                    "[]".to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn typ(
        &mut self,
        pointers: u8,
        name: String,
        arrays: Vec<(Option<usize>, Option<String>)>,
        _funptr_args: Option<Vec<Node>>,
        funptr_rets: Option<Box<Node>>,
    ) {
        if funptr_rets.is_some() {
            panic!("error: function pointers must be named types")
        } else {
            self.w(format_args!(
                "{}{}{}",
                name,
                "*".repeat(pointers.into()),
                CBackend::get_type_array_str(arrays)
            ))
        }
    }

    pub fn typ_from_node(&mut self, node: Node) {
        match node {
            Node::Type {
                pointers,
                name,
                arrays,
                funptr_args,
                funptr_rets,
            } => self.typ(pointers, name, arrays, funptr_args, funptr_rets),
            _ => panic!("named_typ_from_node: node was not of Node::Type"),
        }
    }

    pub fn named_typ(
        &mut self,
        id: String,
        pointers: u8,
        name: String,
        arrays: Vec<(Option<usize>, Option<String>)>,
        funptr_args: Option<Vec<Node>>,
        funptr_rets: Option<Box<Node>>,
    ) {
        if funptr_rets.is_some() {
            self.typ_from_node(*funptr_rets.unwrap());
            self.w(format_args!("(*{} {})(", "*".repeat(pointers.into()), id));
            let funptr_args = funptr_args.unwrap();
            if funptr_args.len() > 0 {
                let end = funptr_args.len() - 1;
                let mut index = 0;
                for arg in funptr_args {
                    self.typ_from_node(arg);
                    if end != index {
                        self.ws(", ")
                    }
                    index += 1
                }
            }
            self.ws(")");
            self.ws(CBackend::get_type_array_str(arrays).as_str());
        } else {
            self.w(format_args!(
                "{} {}{}{}",
                name,
                "*".repeat(pointers.into()),
                id,
                CBackend::get_type_array_str(arrays)
            ))
        }
    }

    pub fn named_typ_from_node(&mut self, id: String, node: Node) {
        match node {
            Node::Type {
                pointers,
                name,
                arrays,
                funptr_args,
                funptr_rets,
            } => self.named_typ(id, pointers, name, arrays, funptr_args, funptr_rets),
            _ => panic!("named_typ_from_node: node was not of Node::Type"),
        }
    }

    // #region: Top level statements

    pub fn top_use(&mut self, path: PathBuf) {
        unimplemented!()
    }

    pub fn top_fun(
        &mut self,
        tags: Vec<FunTags>,
        id: String,
        params: Vec<(String, Node)>,
        rets: Box<Node>,
        expr: Box<Node>,
    ) {
        for hashtag in tags {
            match hashtag {
                FunTags::NoRet => self.ws("noreturn "),
                FunTags::Extern => unimplemented!(),
                FunTags::Inline => self.ws("inline "),
                FunTags::Static => self.ws("static "),
            }
        }

        self.typ_from_node(*rets);
        self.w(format_args!(" {id}("));
        self.compiler.push_scope();
        self.compiler.symbols.add_symbol(id, Symbol::Fun);

        if params.len() > 0 {
            let len = params.len() - 1;
            let mut index = 0;
            for (param_id, param_type) in params {
                self.named_typ_from_node(param_id.to_string(), param_type.clone());
                if index != len {
                    self.ws(", ")
                }
                self.compiler.symbols.add_scoped_symbol(
                    param_id.to_string(),
                    self.compiler.scope,
                    Symbol::FunParam,
                );
                index += 1;
            }
        }

        self.ws(")\n");
        self.write(*expr);
        self.ws("\n");
        self.compiler.pop_scope();
    }

    pub fn top_rec(&mut self, tags: Vec<RecTags>, id: String, fields: Vec<(String, Node)>) {
        let mut is_union = false;
        for hashtag in tags {
            match hashtag {
                RecTags::Union => is_union = true,
                RecTags::Static => self.ws("static "),
            }
        }

        self.ws("typedef ");

        if is_union {
            self.ws("union ");
        } else {
            self.ws("struct ");
        }

        self.ws(id.as_str());

        self.ws("{\n");
        for (field_name, field_type) in fields {
            self.named_typ_from_node(field_name, field_type);
            self.ws(";\n");
        }
        self.w(format_args!("}} {id};\n"));
    }

    // #endregion: Top level statements

    // #region: Statements

    pub fn stat_ret(&mut self, node: Option<Node>) {
        self.ws("return ");
        node.map(|it| self.write(it));
        self.ws(";\n");
    }

    // #endregion Statements

    // #region: Expressions

    pub fn expr_group(&mut self, expr: Node) {
        self.ws("(");
        self.write(expr);
        self.ws(")");
    }

    pub fn expr_number(&mut self, number: String) {
        self.ws(number.as_str());
    }

    pub fn expr_string(&mut self, string: String) {
        self.w(format_args!(
            "(String){{false, {}, \"{}\"}}",
            string.len(),
            string
        ));
    }

    pub fn expr_c_string(&mut self, string: String) {
        self.w(format_args!("\"{}\"", string));
    }

    pub fn expr_char(&mut self, ch: char) {
        self.w(format_args!("'{ch}'"));
    }

    pub fn expr_true(&mut self) {
        self.ws("true");
    }

    pub fn expr_false(&mut self) {
        self.ws("true");
    }

    pub fn expr_id(&mut self, id: String) {
        self.ws(id.as_str());
    }

    pub fn expr_block(&mut self, nodes: Vec<Node>) {
        self.ws("{\n");
        for node in nodes {
            self.write(node);
        }
        self.ws("}");
    }

    pub fn expr_new(&mut self, id: String, params: Vec<Node>) {
        self.w(format_args!("({id}){{"));
        self.comma_separated(params);
        self.ws("}")
    }

    pub fn expr_unary_operator(&mut self, kind: OperatorKind, value: Node) {
        self.ws("(");
        match kind {
            // Postfix
            OperatorKind::Inc | OperatorKind::Dec => {
                self.write(value);
                match kind {
                    OperatorKind::Inc => {
                        self.ws("++");
                    }
                    OperatorKind::Dec => {
                        self.ws("--");
                    }
                    _ => panic!("cannot write as unary operator: {kind:?}"),
                }
            }
            // Prefix
            _ => {
                match kind {
                    OperatorKind::Ref => self.ws("&"),
                    OperatorKind::Deref => self.ws("*"),
                    OperatorKind::Not => self.ws("!"),
                    OperatorKind::Negate => self.ws("-"),
                    _ => panic!("cannot write as unary operator: {kind:?}"),
                }
                self.write(value);
            }
        }
        self.ws(")");
    }

    pub fn expr_binary_operator(&mut self, kind: OperatorKind, left: Node, right: Node) {
        // We need special handling for `as` since the right node has to come before the left node
        if kind == OperatorKind::As {
            self.ws("(");
            self.write(right);
            self.ws(")(");
            self.write(left);
            self.ws(")");
            return;
        }

        self.ws("(");
        self.write(left);
        match kind {
            OperatorKind::Dot => self.ws("."),
            OperatorKind::Assign => self.ws("="),
            OperatorKind::And => self.ws("&&"),
            OperatorKind::Or => self.ws("||"),
            OperatorKind::Eq => self.ws("=="),
            OperatorKind::Neq => self.ws("!="),
            OperatorKind::Gt => self.ws(">"),
            OperatorKind::GtEq => self.ws(">="),
            OperatorKind::Lt => self.ws("<"),
            OperatorKind::LtEq => self.ws("<="),
            OperatorKind::Add => self.ws("+"),
            OperatorKind::Sub => self.ws("-"),
            OperatorKind::Mul => self.ws("*"),
            OperatorKind::Div => self.ws("/"),
            OperatorKind::Mod => self.ws("%"),
            _ => panic!("cannot write as binary operator: {kind:?}"),
        }
        self.write(right);
        self.ws(")");
    }

    pub fn expr_invoke(&mut self, left: Node, params: Vec<Node>) {
        self.ws("(");
        self.write(left);
        self.ws("(");
        if params.len() > 0 {
            let end = params.len() - 1;
            let mut index = 0;
            for param in params {
                self.write(param);
                if index != end {
                    self.ws(", ");
                }
                index += 1;
            }
        }
        self.ws("))");
    }

    pub fn expr_var(&mut self, name: String, typ: Option<Node>, value: Node) {
        match typ {
            Some(typ) => self.named_typ_from_node(name, typ),
            None => panic!("type inference unexpected"),
        }
        self.ws(" = ");
        self.write(value);
    }

    pub fn expr_let(&mut self, name: String, typ: Option<Node>, value: Node) {
        self.ws("const ");
        match typ {
            Some(typ) => self.named_typ_from_node(name, typ),
            None => panic!("type inference unexpected"),
        }
        self.ws(" = ");
        self.write(value);
    }

    // #endregion Expressions
}

impl Backend for CBackend {
    fn write(&mut self, node: Node) {
        match node {
            Node::Program(nodes) => self.program(nodes),
            Node::Raw(text) => self.raw(text),
            Node::Type {
                pointers,
                name,
                arrays,
                funptr_args,
                funptr_rets,
            } => self.typ(pointers, name, arrays, funptr_args, funptr_rets),
            Node::TopUse(path_buf) => self.top_use(path_buf),
            Node::TopFun {
                tags,
                id,
                params,
                rets,
                expr,
            } => self.top_fun(tags, id, params, rets, expr),
            Node::TopRec { tags, id, fields } => self.top_rec(tags, id, fields),
            Node::TopDef { tags, id, typ } => todo!(),
            Node::TopMac {
                tags,
                id,
                params,
                rets,
                expands_to,
            } => todo!(),
            Node::TopTag { tags, id, entries } => todo!(),
            Node::TopTagRec { tags, id, entries } => todo!(),
            Node::StatRet(node) => match node {
                Some(it) => self.stat_ret(Some(*it)),
                None => self.stat_ret(None),
            },
            Node::StatIf { cond, expr, else_ } => todo!(),
            Node::StatSwitch { switch, cases } => todo!(),
            Node::StatForCStyle {
                def,
                cond,
                inc,
                expr,
            } => todo!(),
            Node::StatForSingleExpr { cond, expr } => todo!(),
            Node::StatForRange {
                var,
                from,
                to,
                expr,
            } => todo!(),
            Node::StatExpr(node) => {
                self.write(*node);
                self.ws(";\n");
            }
            Node::ExprGroup(node) => self.expr_group(*node),
            Node::ExprNumber(number) => self.expr_number(number),
            Node::ExprString(string) => self.expr_string(string),
            Node::ExprCString(string) => self.expr_c_string(string),
            Node::ExprChar(ch) => self.expr_char(ch),
            Node::ExprTrue => self.expr_true(),
            Node::ExprFalse => self.expr_false(),
            Node::ExprIdentifier(id) => self.expr_id(id),
            Node::ExprBlock(nodes) => self.expr_block(nodes),
            Node::ExprNew { id, params } => self.expr_new(id, params),
            Node::ExprUnaryOperator { kind, value } => self.expr_unary_operator(kind, *value),
            Node::ExprBinaryOperator { kind, left, right } => {
                self.expr_binary_operator(kind, *left, *right)
            }
            Node::ExprInvoke { left, params } => self.expr_invoke(*left, params),
            Node::ExprMacInvoke { name, params } => todo!(),
            Node::ExprList(nodes) => todo!(),
            Node::ExprVar { name, typ, value } => match typ {
                Some(it) => self.expr_var(name, Some(*it), *value),
                None => self.expr_var(name, None, *value),
            },
            Node::ExprLet { name, typ, value } => match typ {
                Some(it) => self.expr_let(name, Some(*it), *value),
                None => self.expr_let(name, None, *value),
            },
        }
    }
}

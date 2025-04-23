use core::{fmt, panic};
use std::{io::Write, path::PathBuf};

use crate::{
    backend::backend::Backend,
    compile::{compiler::Compiler, error::CompilerError, symbol::Symbol},
    hashtags::{DefTags, FunTags, RecTags, TagRecTags, TagTags},
    parse::{
        ast::{Node, NodeKind},
        operator::OperatorKind,
    },
};

pub struct CBackend<'a> {
    pub node: Box<Node>, // reference to current node
    pub compiler: Compiler<'a>,
}

impl<'a> CBackend<'a> {
    pub fn new(compiler: Compiler<'a>) -> Self {
        CBackend {
            node: Box::new(Node {
                line: 0,
                column: 0,
                node: NodeKind::Raw(Default::default()),
            }),
            compiler,
        }
    }

    pub fn throw(&self, error: CompilerError, help: Option<&str>) -> ! {
        self.compiler
            .throw_exception(error, help, *self.node.clone())
    }

    pub fn get_symbol(&self, symbol: String) -> Option<&Symbol> {
        self.compiler.symbols.get_symbol(symbol)
    }

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
        match node.node {
            NodeKind::Type {
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
        match node.node {
            NodeKind::Type {
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
        self.compiler.add_fun(id);

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
                    Symbol::Var,
                );
                index += 1;
            }
        }

        self.ws(")\n");
        self.write(*expr);
        self.ws("\n\n");
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
            self.ws("\t");
            self.named_typ_from_node(field_name, field_type);
            self.ws(";\n");
        }
        self.w(format_args!("}} {id};\n\n"));

        self.compiler.add_rec(id);
    }

    pub fn top_def(&mut self, tags: Vec<DefTags>, id: String, typ: Node) {
        for hashtag in tags {
            match hashtag {
                DefTags::Static => self.ws("static "),
            }
        }

        self.ws("typedef ");
        self.typ_from_node(typ);
        self.w(format_args!("{id};\n\n"));

        self.compiler.add_def(id);
    }

    pub fn top_tag(&mut self, tags: Vec<TagTags>, id: String, entries: Vec<String>) {
        for hashtag in tags {
            match hashtag {
                TagTags::Static => self.ws("static "),
            }
        }

        self.ws("typedef enum {\n");
        for entry in entries {
            self.w(format_args!("\t{entry},\n"));
        }
        self.w(format_args!("}} {id};\n\n"));

        self.compiler.add_tag(id);
    }

    pub fn top_tag_rec(
        &mut self,
        tags: Vec<TagRecTags>,
        id: String,
        entries: Vec<(String, Vec<(String, Node)>)>,
    ) {
        let is_static = tags.contains(&TagRecTags::Static);

        if is_static {
            self.ws("static ");
        }
        self.ws("typedef enum {\n");
        for (entry_id, _entry_fields) in &entries {
            self.w(format_args!("\t{entry_id},\n"))
        }
        self.w(format_args!("}} _{id}_tag;\n\n"));

        if is_static {
            self.ws("static ");
        }
        self.ws("typedef struct {\n");
        self.w(format_args!("\t_{id}_tag kind;\n"));
        self.ws("\tunion {\n");
        for (entry_id, entry_fields) in &entries {
            self.ws("\t\tstruct { ");
            for (field, typ) in entry_fields {
                // self.ws("");
                self.named_typ_from_node(field.to_string(), typ.clone());
                self.ws("; ");
            }
            self.w(format_args!("}} {entry_id};\n"));
        }
        self.ws("\t};\n");
        self.w(format_args!("}} {id};\n\n"));

        self.compiler.add_tag_rec(id);
    }

    // #endregion: Top level statements

    // #region: Statements

    pub fn stat_ret(&mut self, node: Option<Node>) {
        self.ws("return ");
        node.map(|it| self.write(it));
        self.ws(";\n");
    }

    pub fn stat_if(&mut self, cond: Node, expr: Node, else_: Option<Node>) {
        self.ws("if (");
        self.write(cond);
        self.ws(") {");
        self.write(expr);
        self.ws("}");
        if let Some(else_) = else_ {
            self.ws(" else {");
            self.write(else_);
            self.ws("}");
        }
    }

    pub fn stat_switch(&mut self, switch: Node, cases: Vec<(Option<Box<Node>>, bool, Box<Node>)>) {
        self.ws("switch (");
        self.write(switch);
        self.ws(") {\n");
        for (case, fall, expr) in cases {
            match case {
                Some(case) => {
                    self.ws("case (");
                    self.write(*case);
                    self.ws("): "); // aww it's sad :(
                }
                None => {
                    self.ws("default: "); // this one isn't sad :)
                }
            }
            self.ws("{");
            self.write(*expr);
            if fall {
                self.ws("break;");
            }
            self.ws("}")
        }
        self.ws("}\n");
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
        let symbol = self.get_symbol(id.clone());

        if symbol.is_some_and(|it| it.instantiatable()) {
            // When instantiating tag recs, we want to explicitly specify which union we are instantiating
            if symbol.unwrap() == &Symbol::TagRec {
                if params.len() == 0 {
                    self.throw(CompilerError::TagRecInstantiateWithoutKind, None);
                }
                let kind = &params[0];
                let kind_str = match &kind.node {
                    NodeKind::ExprIdentifier(id) => id,
                    _ => self.throw(
                        CompilerError::TagRecInstantiateWithoutKind,
                        Some("the first parameter must be an entry in the tag rec (it currently is not)"),
                    ),
                };
                self.write(kind.clone());
                if params.len() > 1 {
                    self.w(format_args!(", .{kind_str}={{"));
                    self.comma_separated(params[1..].to_vec());
                    self.ws("}");
                }
            } else {
                self.comma_separated(params);
            }
            self.ws("}")
        } else {
            if symbol.is_none() {
                self.throw(CompilerError::UnknownSymbol(id), None);
            } else {
                self.throw(
                    CompilerError::Uninstantiatable(id, symbol.unwrap().clone()),
                    None,
                );
            }
        }
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

impl<'a> Backend for CBackend<'a> {
    fn write(&mut self, node: Node) {
        self.node = Box::new(node.clone());
        match node.node {
            NodeKind::Program(nodes) => self.program(nodes),
            NodeKind::Raw(text) => self.raw(text),
            NodeKind::Type {
                pointers,
                name,
                arrays,
                funptr_args,
                funptr_rets,
            } => self.typ(pointers, name, arrays, funptr_args, funptr_rets),
            NodeKind::TopUse(path_buf) => self.top_use(path_buf),
            NodeKind::TopFun {
                tags,
                id,
                params,
                rets,
                expr,
            } => self.top_fun(tags, id, params, rets, expr),
            NodeKind::TopRec { tags, id, fields } => self.top_rec(tags, id, fields),
            NodeKind::TopDef { tags, id, typ } => self.top_def(tags, id, *typ),
            NodeKind::TopMac {
                tags,
                id,
                params,
                rets,
                expands_to,
            } => todo!(),
            NodeKind::TopTag { tags, id, entries } => self.top_tag(tags, id, entries),
            NodeKind::TopTagRec { tags, id, entries } => self.top_tag_rec(tags, id, entries),
            NodeKind::StatRet(node) => self.stat_ret(node.map(|it| *it)),
            NodeKind::StatIf { cond, expr, else_ } => {
                self.stat_if(*cond, *expr, else_.map(|it| *it))
            }
            NodeKind::StatSwitch { switch, cases } => self.stat_switch(*switch, cases),
            NodeKind::StatForCStyle {
                def,
                cond,
                inc,
                expr,
            } => todo!(),
            NodeKind::StatForSingleExpr { cond, expr } => todo!(),
            NodeKind::StatForRange {
                var,
                from,
                to,
                expr,
            } => todo!(),
            NodeKind::StatExpr(node) => {
                self.write(*node);
                self.ws(";\n");
            }
            NodeKind::ExprGroup(node) => self.expr_group(*node),
            NodeKind::ExprNumber(number) => self.expr_number(number),
            NodeKind::ExprString(string) => self.expr_string(string),
            NodeKind::ExprCString(string) => self.expr_c_string(string),
            NodeKind::ExprChar(ch) => self.expr_char(ch),
            NodeKind::ExprTrue => self.expr_true(),
            NodeKind::ExprFalse => self.expr_false(),
            NodeKind::ExprIdentifier(id) => self.expr_id(id),
            NodeKind::ExprBlock(nodes) => self.expr_block(nodes),
            NodeKind::ExprNew { id, params } => self.expr_new(id, params),
            NodeKind::ExprUnaryOperator { kind, value } => self.expr_unary_operator(kind, *value),
            NodeKind::ExprBinaryOperator { kind, left, right } => {
                self.expr_binary_operator(kind, *left, *right)
            }
            NodeKind::ExprInvoke { left, params } => self.expr_invoke(*left, params),
            NodeKind::ExprMacInvoke { name, params } => todo!(),
            NodeKind::ExprList(nodes) => todo!(),
            NodeKind::ExprVar { name, typ, value } => {
                self.expr_var(name, typ.map(|it| *it), *value)
            }
            NodeKind::ExprLet { name, typ, value } => {
                self.expr_let(name, typ.map(|it| *it), *value)
            }
        }
    }
}

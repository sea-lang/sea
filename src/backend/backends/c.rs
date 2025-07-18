use core::{fmt, format_args, panic};
use std::{fs, io::Write, path::PathBuf};

use crate::{
    backend::backend::Backend,
    compile::{
        compiler::Compiler, error::CompilerError, infer::infer_type_of_node, symbol::Symbol,
        type_::SeaType,
    },
    hashtags::{DefTags, FunTags, RecTags, TagRecTags, TagTags},
    parse::{
        ast::{Node, NodeKind},
        lexer::Lexer,
        operator::OperatorKind,
        parser::Parser,
    },
};

#[derive(Clone)]
pub struct CodeBlock {
    pub deferred: Vec<Node>,
    pub returned: bool,
}

pub struct CBackend<'a, 'b> {
    pub node: Box<Node>, // reference to current node
    pub compiler: &'b mut Compiler<'a>,
    pub block_stack: Vec<CodeBlock>,
}

impl<'a, 'b> CBackend<'a, 'b> {
    const NAMESPACE_SEP: &'static str = "$";

    pub fn new(compiler: &'b mut Compiler<'a>) -> Self {
        CBackend {
            node: Box::new(Node {
                line: 0,
                column: 0,
                node: NodeKind::Raw(Default::default()),
            }),
            compiler,
            block_stack: vec![],
        }
    }

    pub fn throw(&self, error: CompilerError, help: Option<&str>) -> ! {
        self.compiler.throw(error, help, *self.node.clone())
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
        let file_path = self
            .compiler
            .file_stack
            .last()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        self.w(format_args!("#pragma region \"file: {file_path}\"\n"));

        for node in program {
            self.write(node);
        }

        self.w(format_args!("#pragma region \"end file: {file_path}\"\n"));
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
                name.replace('\'', Self::NAMESPACE_SEP),
                "*".repeat(pointers.into()),
                CBackend::get_type_array_str(arrays)
            ))
        }
    }

    pub fn typ_from_seatype(&mut self, typ: SeaType) {
        if typ.funptr_rets.is_some() {
            panic!("error: function pointers must be named types")
        } else {
            self.w(format_args!(
                "{}{}{}",
                typ.name.replace('\'', Self::NAMESPACE_SEP),
                "*".repeat(typ.pointers.into()),
                CBackend::get_type_array_str(typ.arrays)
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
            self.w(format_args!(
                "(*{} {})(",
                "*".repeat(pointers.into()),
                id.replace('\'', Self::NAMESPACE_SEP)
            ));
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
                name.replace('\'', Self::NAMESPACE_SEP),
                "*".repeat(pointers.into()),
                id.replace('\'', Self::NAMESPACE_SEP),
                CBackend::get_type_array_str(arrays)
            ))
        }
    }

    pub fn named_typ_from_seatype(&mut self, id: String, typ: SeaType) {
        if typ.funptr_rets.is_some() {
            self.typ_from_seatype(*typ.funptr_rets.unwrap());
            self.w(format_args!(
                "(*{} {})(",
                "*".repeat(typ.pointers.into()),
                id.replace('\'', Self::NAMESPACE_SEP)
            ));
            let funptr_args = typ.funptr_args.unwrap();
            if funptr_args.len() > 0 {
                let end = funptr_args.len() - 1;
                let mut index = 0;
                for arg in funptr_args {
                    self.typ_from_seatype(arg);
                    if end != index {
                        self.ws(", ")
                    }
                    index += 1
                }
            }
            self.ws(")");
            self.ws(CBackend::get_type_array_str(typ.arrays).as_str());
        } else {
            self.w(format_args!(
                "{} {}{}{}",
                typ.name.replace('\'', Self::NAMESPACE_SEP),
                "*".repeat(typ.pointers.into()),
                id.replace('\'', Self::NAMESPACE_SEP),
                CBackend::get_type_array_str(typ.arrays)
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

    pub fn write_deferred(&mut self, all: bool, clear: bool) {
        if all {
            // I almost definitely shouldn't be cloning this here but whatever.
            self.block_stack.clone().iter().rev().for_each(|block| {
                block.deferred.clone().iter().for_each(|it| {
                    self.ws("/* deferred */\n"); // write an indicator that this code block is deferred
                    self.write(it.clone());
                })
            });

            if clear {
                // Consume all of the deferred blocks
                self.block_stack.iter_mut().for_each(|block| {
                    block.deferred.clear();
                });
            }
        } else {
            // I probably shouldn't be cloning this here but whatever.
            self.block_stack
                .last()
                .unwrap()
                .deferred
                .clone()
                .iter()
                .for_each(|it| {
                    self.ws("/* deferred */\n"); // write an indicator that this code block is deferred
                    self.write(it.clone());
                });

            if clear {
                // Consume the deferred block
                self.block_stack.last_mut().unwrap().deferred.clear();
            }
        }
    }

    // #region: Top level statements

    pub fn top_use(&mut self, path: PathBuf) {
        let file_paths = self.compiler.get_use_paths(path.clone());
        if let Ok(file_paths) = file_paths {
            for path in file_paths {
                if !path.exists() {
                    self.throw(
                        CompilerError::ImportError(format!("no such module: {path:?}")),
                        None,
                    );
                }

                if self.compiler.uses(&path) {
                    continue;
                }

                self.compiler.file_stack.push(path.clone());

                self.compiler.usages.push(path.clone());
                let code = fs::read_to_string(path.clone()).unwrap();
                let mut parser = Parser::new(Lexer::new(path, &code));
                let program = parser.parse(false);
                self.write(program);

                self.compiler.file_stack.pop();
            }
        } else {
            self.throw(CompilerError::ImportError(file_paths.err().unwrap()), None)
        }
    }

    pub fn top_pkg(&mut self, namespace: String, statements: Vec<Node>) {
        for node in statements {
            self.pkg_statement(node, &namespace);
        }
    }

    pub fn top_fun(
        &mut self,
        tags: Vec<FunTags>,
        id: String,
        params: Vec<(String, Node)>,
        rets: Box<Node>,
        expr: Box<Node>,
    ) {
        for hashtag in tags.clone() {
            match hashtag {
                FunTags::NoRet => self.ws("noreturn "), // TODO: is noreturn compiler-specific?
                FunTags::Extern => unimplemented!(),
                FunTags::Inline => self.ws("inline "),
                FunTags::Static => self.ws("static "),
            }
        }

        self.typ_from_node((*rets).clone());
        self.w(format_args!(" {}(", id.replace('\'', Self::NAMESPACE_SEP)));
        self.compiler.push_scope();

        self.compiler.add_fun(
            id,
            tags,
            params
                .iter()
                .map(|(_, it)| SeaType::from_node(it.clone()).unwrap())
                .collect::<Vec<SeaType>>(),
            SeaType::from_node(*rets).unwrap(),
        );

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
                    Symbol::Var {
                        typ: SeaType::from_node(param_type).unwrap(),
                        mutable: true,
                    },
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
        for hashtag in tags.clone() {
            match hashtag {
                RecTags::Union => is_union = true,
                RecTags::Static => self.ws("static "), //todo: can structs even be static??? is it ever used if so???
            }
        }

        // Write a forward declaration
        self.ws("typedef ");
        self.ws(if is_union { "union " } else { "struct " });
        self.ws(id.replace('\'', Self::NAMESPACE_SEP).as_str());
        self.ws(" ");
        self.ws(id.replace('\'', Self::NAMESPACE_SEP).as_str());
        self.ws(";\n");

        // Write the struct declaration
        self.ws("typedef ");
        self.ws(if is_union { "union " } else { "struct " });
        self.ws(id.replace('\'', Self::NAMESPACE_SEP).as_str());
        self.ws("{\n");
        for (field_name, field_type) in &fields {
            self.ws("\t");
            self.named_typ_from_node(field_name.clone(), field_type.clone());
            self.ws(";\n");
        }
        self.w(format_args!(
            "}} {};\n\n",
            id.replace('\'', Self::NAMESPACE_SEP)
        ));

        self.compiler.add_rec(
            id,
            tags,
            fields
                .iter()
                .map(|(name, typ)| (name.clone(), SeaType::from_node(typ.clone()).unwrap()))
                .collect::<Vec<(String, SeaType)>>(),
        );
    }

    pub fn top_def(&mut self, tags: Vec<DefTags>, id: String, typ: Node) {
        for hashtag in tags.clone() {
            match hashtag {
                DefTags::Static => self.ws("static "),
            }
        }

        self.ws("typedef ");
        self.named_typ_from_node(id.replace('\'', Self::NAMESPACE_SEP), typ.clone());
        self.ws(";\n\n");

        self.compiler
            .add_def(id, tags, SeaType::from_node(typ).unwrap());
    }

    pub fn top_tag(
        &mut self,
        tags: Vec<TagTags>,
        id: String,
        entries: Vec<(String, Option<Box<Node>>)>,
    ) {
        let mut skip_helpers = false;
        for hashtag in tags.clone() {
            match hashtag {
                TagTags::Static => self.ws("static "),
                TagTags::NoHelpers => skip_helpers = true,
            }
        }

        let namespaced_id = id.replace('\'', Self::NAMESPACE_SEP);
        self.ws("typedef enum {\n");
        for (entry, value) in &entries {
            self.w(format_args!(
                "\t{namespaced_id}{}{entry}",
                Self::NAMESPACE_SEP
            ));
            if value.is_some() {
                self.ws(" = ");
                self.write(*value.clone().unwrap());
            }
            self.ws(",\n");
        }
        self.w(format_args!("}} {namespaced_id};\n\n"));

        self.compiler.add_tag(
            id.clone(),
            tags,
            entries
                .iter()
                .map(|it| it.0.clone())
                .collect::<Vec<String>>(),
        );

        if entries.len() == 0 || skip_helpers {
            return;
        }

        // Create the `entries` array
        self.w(format_args!(
            "const {} {}{}{}[] = {{\n",
            namespaced_id,
            namespaced_id,
            Self::NAMESPACE_SEP,
            "entries"
        ));
        entries.iter().for_each(|it| {
            self.w(format_args!(
                "\t{}{}{},\n",
                namespaced_id,
                Self::NAMESPACE_SEP,
                it.0
            ));
        });
        self.ws("};\n\n");

        self.w(format_args!(
            "const int {}{}len = {};\n\n",
            namespaced_id,
            Self::NAMESPACE_SEP,
            entries.len()
        ));

        // to_str and from_str functions. I hope you're ready for some truly horrendous code
        // to_str
        let to_str_cases = entries
            .iter()
            .map(|it| {
                (
                    Some(Box::new(Node::of_kind(NodeKind::ExprIdentifier(
                        format_args!("{}{}{}", id, Self::NAMESPACE_SEP, it.0).to_string(),
                    )))),
                    false,
                    Box::new(Node::of_kind(NodeKind::ExprBlock(vec![Node::of_kind(
                        NodeKind::StatRet(Some(Box::new(Node::of_kind(NodeKind::ExprString(
                            it.0.clone(),
                        ))))),
                    )]))),
                )
            })
            .collect::<Vec<(Option<Box<Node>>, bool, Box<Node>)>>();
        self.top_fun(
            vec![],
            format_args!("{}'to_str", id.clone()).to_string(),
            vec![(
                "it".to_string(),
                Node::of_kind(NodeKind::Type {
                    pointers: 0,
                    name: id.clone(),
                    arrays: vec![],
                    funptr_args: None,
                    funptr_rets: None,
                }),
            )],
            Box::new(Node::of_kind(NodeKind::Type {
                pointers: 0,
                name: "String".to_string(),
                arrays: vec![],
                funptr_args: None,
                funptr_rets: None,
            })),
            Box::new(Node::of_kind(NodeKind::ExprBlock(vec![
                Node::of_kind(NodeKind::StatSwitch {
                    switch: Box::new(Node::of_kind(NodeKind::ExprIdentifier("it".to_string()))),
                    cases: to_str_cases,
                }),
                Node::of_kind(NodeKind::StatRet(Some(Box::new(Node::of_kind(
                    NodeKind::ExprString("".to_string()),
                ))))),
            ]))),
        );

        // from_str
        if !self.compiler.uses(&PathBuf::from("std/str")) {
            // If `std` is in usage, then we can probably add std/str implicitly.
            // If it *isn't* in usage then the user is likely using --nostd.
            if self.compiler.uses(&PathBuf::from("std")) {
                self.top_use(PathBuf::from("std/str"))
            } else {
                return;
            }
        }
        let make_if_cond_for = |entry: String, else_: NodeKind| NodeKind::StatIf {
            cond: Box::new(Node::of_kind(NodeKind::ExprInvoke {
                left: Box::new(Node::of_kind(NodeKind::ExprIdentifier(
                    "str'compare".to_string(),
                ))),
                params: vec![
                    Node::of_kind(NodeKind::ExprIdentifier("it".to_string())),
                    Node::of_kind(NodeKind::ExprString(entry.clone())),
                ],
            })),
            expr: Box::new(Node::of_kind(NodeKind::ExprBlock(vec![Node::of_kind(
                NodeKind::StatRet(Some(Box::new(Node::of_kind(NodeKind::ExprIdentifier(
                    format_args!("{id}'{entry}").to_string(),
                ))))),
            )]))),
            else_: Some(Box::new(Node::of_kind(else_))),
        };
        let mut from_str_if_chain =
            NodeKind::StatRet(Some(Box::new(Node::of_kind(NodeKind::ExprIdentifier(
                format_args!("{}{}{}", id, Self::NAMESPACE_SEP, entries.last().unwrap().0)
                    .to_string(),
            )))));
        entries.iter().for_each(|it| {
            from_str_if_chain = make_if_cond_for(it.0.clone(), from_str_if_chain.clone())
        });
        self.top_fun(
            vec![],
            format_args!("{}'from_str", id.clone()).to_string(),
            vec![(
                "it".to_string(),
                Node::of_kind(NodeKind::Type {
                    pointers: 0,
                    name: "String".to_string(),
                    arrays: vec![],
                    funptr_args: None,
                    funptr_rets: None,
                }),
            )],
            Box::new(Node::of_kind(NodeKind::Type {
                pointers: 0,
                name: id.clone(),
                arrays: vec![],
                funptr_args: None,
                funptr_rets: None,
            })),
            Box::new(Node::of_kind(NodeKind::ExprBlock(vec![Node::of_kind(
                from_str_if_chain,
            )]))),
        );
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
        let namespaced_id = id.replace('\'', Self::NAMESPACE_SEP);
        for (entry_id, _entry_fields) in &entries {
            self.w(format_args!(
                "\t{}{}{},\n",
                namespaced_id,
                Self::NAMESPACE_SEP,
                entry_id
            ))
        }
        self.w(format_args!("}} _{namespaced_id}_tag;\n\n"));

        let mut mapped_entries: Vec<(String, Vec<(String, SeaType)>)> = vec![];

        for (entry_id, entry_fields) in &entries {
            self.ws("typedef struct { ");
            let name = format!("_{namespaced_id}_{entry_id}");
            for (field, typ) in entry_fields {
                self.named_typ_from_node(field.to_string(), typ.clone());
                self.ws("; ");
            }
            self.w(format_args!("}} {name};\n"));
            let mapped_fields = entry_fields
                .iter()
                .map(|(field_name, field_typ)| {
                    (
                        field_name.clone(),
                        SeaType::from_node(field_typ.clone()).unwrap(),
                    )
                })
                .collect::<Vec<(String, SeaType)>>();
            mapped_entries.push((entry_id.clone(), mapped_fields.clone()));
            self.compiler.add_rec(name, vec![], mapped_fields);
        }

        if is_static {
            self.ws("static ");
        }
        self.ws("typedef struct {\n");
        self.w(format_args!("\t_{namespaced_id}_tag kind;\n"));
        self.ws("\tunion {\n");
        for (entry_id, _) in &entries {
            self.w(format_args!(
                "\t\t_{namespaced_id}_{entry_id} {entry_id};\n"
            ));
        }
        self.ws("\t};\n");
        self.w(format_args!("}} {namespaced_id};\n\n"));

        self.compiler.add_tag_rec(id, tags, mapped_entries);
    }

    // #endregion: Top level statements

    // #region: Statements

    pub fn stat_ret(&mut self, node: Option<Node>) {
        self.write_deferred(true, false);
        self.ws("return ");
        node.map(|it| self.write(it));
        self.ws(";\n");
        self.block_stack.last_mut().unwrap().returned = true;
    }

    pub fn stat_if(&mut self, cond: Node, expr: Node, else_: Option<Node>) {
        self.ws("if (");
        self.write(cond);
        self.ws(") {");
        self.write(expr);
        self.ws("}");
        if let Some(else_) = else_ {
            match else_.node {
                NodeKind::StatIf {
                    cond: _,
                    expr: _,
                    else_: _,
                } => {
                    self.ws(" else ");
                    self.write(else_);
                }
                _ => {
                    self.ws(" else {");
                    self.write(else_);
                    self.ws("}");
                }
            }
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
                _ => {
                    self.ws("default: "); // this one isn't sad :)
                }
            }
            self.ws("{");
            self.write(*expr);
            if !fall {
                self.ws("break;");
            }
            self.ws("}\n")
        }
        self.ws("}\n");
    }

    pub fn stat_for_c_style(&mut self, def: Node, cond: Node, inc: Node, expr: Node) {
        self.ws("for (");
        self.write(def);
        self.ws(" ; ");
        self.write(cond);
        self.ws(" ; ");
        self.write(inc);
        self.ws(") {");
        self.write(expr);
        self.ws("}")
    }

    pub fn stat_for_single_expr(&mut self, cond: Node, expr: Node) {
        self.ws("while (");
        self.write(cond);
        self.ws(") {");
        self.write(expr);
        self.ws("}")
    }

    pub fn stat_for_range(&mut self, var: Option<String>, from: Node, to: Node, expr: Node) {
        self.ws("for (");
        let v = var.unwrap_or_else(|| "_i".to_string());
        self.w(format_args!("int {v} = "));
        self.write(from);
        self.ws(" ; ");
        self.w(format_args!("{v} < "));
        self.write(to);
        self.w(format_args!(" ; {v}++"));
        self.ws(") {");
        self.write(expr);
        self.ws("}")
    }

    pub fn stat_defer(&mut self, expr: Node) {
        self.block_stack.last_mut().unwrap().deferred.push(expr)
    }

    // #endregion Statements

    // #region: Expressions

    pub fn expr_group(&mut self, expr: Node) {
        self.ws("(");
        self.write(expr);
        self.ws(")");
    }

    pub fn expr_number(&mut self, number: String) {
        self.ws(&number.as_str().replace('_', ""));
    }

    pub fn expr_string(&mut self, string: String) {
        self.w(format_args!(
            "(String){{false, {}, hash{}wyhash{}hash_c_string(_internal{}strsecret, \"{}\", {}), \"{}\"}}",
            string.len(),
            Self::NAMESPACE_SEP,
            Self::NAMESPACE_SEP,
            Self::NAMESPACE_SEP,
            string,
            string.len(),
            string
        ));
    }

    pub fn expr_c_string(&mut self, string: String) {
        self.w(format_args!("\"{}\"", string));
    }

    pub fn expr_char(&mut self, ch: String) {
        self.w(format_args!("'{ch}'"));
    }

    pub fn expr_true(&mut self) {
        self.ws("true");
    }

    pub fn expr_false(&mut self) {
        self.ws("false");
    }

    pub fn expr_id(&mut self, id: String) {
        self.ws(id.replace('\'', Self::NAMESPACE_SEP).as_str());
    }

    pub fn expr_block(&mut self, nodes: Vec<Node>) {
        self.ws("{\n");
        self.block_stack.push(CodeBlock {
            deferred: vec![],
            returned: false,
        });
        for node in nodes {
            self.write(node);
        }
        if !self.block_stack.last().unwrap().returned {
            self.write_deferred(false, true);
        }
        self.block_stack.pop();
        self.ws("}");
    }

    pub fn expr_new(&mut self, id: String, params: Vec<Node>) {
        self.w(format_args!(
            "({}){{",
            id.replace('\'', Self::NAMESPACE_SEP)
        ));
        let symbol = self.get_symbol(id.clone());

        if symbol.is_some_and(|it| it.instantiatable()) {
            // When instantiating tag recs, we want to explicitly specify which union we are instantiating
            match symbol.unwrap() {
                Symbol::TagRec {
                    tags: _,
                    entries: _,
                } => {
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
                        let field = kind_str.split('\'').last().unwrap();
                        self.w(format_args!(", .{field}={{",));
                        self.comma_separated(params[1..].to_vec());
                        self.ws("}");
                    }
                }
                _ => {
                    self.comma_separated(params);
                }
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

        if kind != OperatorKind::Index {
            self.ws("(");
        }
        self.write(left);

        match kind {
            OperatorKind::Dot => self.ws("."),
            OperatorKind::Pkg => self.ws("__"),
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
            OperatorKind::Index => self.ws("["),
            _ => panic!("cannot write as binary operator: {kind:?}"),
        }

        self.write(right);

        if kind == OperatorKind::Index {
            self.ws("]");
        } else {
            self.ws(")");
        }
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

    pub fn expr_list(&mut self, nodes: Vec<Node>) {
        self.ws("{");
        self.comma_separated(nodes);
        self.ws("}");
    }

    pub fn expr_var(&mut self, name: String, typ: Option<Node>, value: Node) {
        let seatyp = match typ {
            Some(typ) => {
                self.named_typ_from_node(name.clone(), typ.clone());
                SeaType::from_node(typ).unwrap()
            }
            _ => match infer_type_of_node(&self.compiler, &value) {
                Ok(it) => {
                    self.named_typ_from_seatype(name.clone(), it.clone());
                    it
                }
                Err(why) => self.throw(CompilerError::InferenceError(why), None),
            },
        };
        self.ws(" = ");
        self.write(value);
        self.compiler.add_var(name, seatyp, true);
    }

    pub fn expr_let(&mut self, name: String, typ: Option<Node>, value: Node) {
        self.ws("const ");
        let seatyp = match typ {
            Some(typ) => {
                self.named_typ_from_node(name.clone(), typ.clone());
                SeaType::from_node(typ).unwrap()
            }
            _ => {
                let t = infer_type_of_node(&self.compiler, &value).unwrap();
                self.named_typ_from_seatype(name.clone(), t.clone());
                t
            }
        };
        self.ws(" = ");
        self.write(value);
        self.compiler.add_var(name, seatyp, false);
    }

    // #endregion Expressions

    pub fn pkg_statement(&mut self, node: Node, namespace: &str) {
        self.node = Box::new(node.clone());
        match node.node {
            NodeKind::Raw(text) => self.raw(text),
            NodeKind::TopPkg { name, statements } => {
                self.top_pkg(format_args!("{namespace}'{name}").to_string(), statements)
            }
            NodeKind::TopFun {
                tags,
                id,
                params,
                rets,
                expr,
            } => self.top_fun(
                tags,
                format_args!("{namespace}'{id}").to_string(),
                params,
                rets,
                expr,
            ),
            NodeKind::TopRec { tags, id, fields } => {
                self.top_rec(tags, format_args!("{namespace}'{id}").to_string(), fields)
            }
            NodeKind::TopDef { tags, id, typ } => {
                self.top_def(tags, format_args!("{namespace}'{id}").to_string(), *typ)
            }
            NodeKind::TopTag { tags, id, entries } => {
                self.top_tag(tags, format_args!("{namespace}'{id}").to_string(), entries)
            }
            NodeKind::TopTagRec { tags, id, entries } => {
                self.top_tag_rec(tags, format_args!("{namespace}'{id}").to_string(), entries)
            }
            NodeKind::StatExpr(expr) => {
                self.pkg_statement(*expr, namespace);
                self.ws(";\n");
            }
            NodeKind::ExprVar { name, typ, value } => self.expr_var(
                format_args!("{namespace}'{name}").to_string(),
                typ.map(|it| *it),
                *value,
            ),
            NodeKind::ExprLet { name, typ, value } => self.expr_let(
                format_args!("{namespace}'{name}").to_string(),
                typ.map(|it| *it),
                *value,
            ),
            _ => self.throw(CompilerError::StatementNotAllowedAtTopLevel(), None),
        }
    }
}

impl<'a, 'b> Backend for CBackend<'a, 'b> {
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
            NodeKind::TopPkg { name, statements } => self.top_pkg(name, statements),
            NodeKind::TopFun {
                tags,
                id,
                params,
                rets,
                expr,
            } => self.top_fun(tags, id, params, rets, expr),
            NodeKind::TopRec { tags, id, fields } => self.top_rec(tags, id, fields),
            NodeKind::TopDef { tags, id, typ } => self.top_def(tags, id, *typ),
            NodeKind::TopTag { tags, id, entries } => self.top_tag(tags, id, entries),
            NodeKind::TopTagRec { tags, id, entries } => self.top_tag_rec(tags, id, entries),
            NodeKind::TopPragma { id: _, params: _ } => self.compiler.handle_pragma(node),
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
            } => self.stat_for_c_style(*def, *cond, *inc, *expr),
            NodeKind::StatForSingleExpr { cond, expr } => self.stat_for_single_expr(*cond, *expr),
            NodeKind::StatForRange {
                var,
                from,
                to,
                expr,
            } => self.stat_for_range(var, *from, *to, *expr),
            NodeKind::StatContinue => {
                self.write_deferred(false, false);
                self.ws("continue;");
            }
            NodeKind::StatBreak => {
                self.write_deferred(false, false);
                self.ws("break;");
            }
            NodeKind::StatDefer(expr) => self.stat_defer(*expr),
            NodeKind::StatExpr(node) => {
                // Get ready for some spaghetti.
                // This checks if the expression is a function invoke, and if so
                // we then check if it has the #noret tag. If so, we will write
                // any deferred statements.
                match &node.node {
                    NodeKind::ExprInvoke { left, params: _ } => match &left.node {
                        NodeKind::ExprIdentifier(id) => {
                            let s = self.compiler.symbols.get_symbol(id.clone());
                            match s {
                                Some(it) => match it {
                                    Symbol::Fun {
                                        tags,
                                        params: _,
                                        rets: _,
                                    } => {
                                        if tags.contains(&FunTags::NoRet) {
                                            self.write_deferred(true, false);
                                        }
                                    }
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }

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
            NodeKind::ExprList(nodes) => self.expr_list(nodes),
            NodeKind::ExprVar { name, typ, value } => {
                self.expr_var(name, typ.map(|it| *it), *value)
            }
            NodeKind::ExprLet { name, typ, value } => {
                self.expr_let(name, typ.map(|it| *it), *value)
            }
        }
    }
}

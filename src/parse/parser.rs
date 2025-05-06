use std::{fmt::Debug, path::PathBuf, process::exit, str::FromStr};

use crate::{
    hashtags::{DefTags, FunTags, RecTags, TagRecTags, TagTags},
    parse::operator::Associativity,
};

use super::{
    ast::{Node, NodeKind},
    error::ParseError,
    lexer,
    operator::OperatorKind,
    token::{Token, TokenKind},
};

pub struct Parser<'a> {
    pub lexer: lexer::Lexer<'a>,
    pub token: Token, // current token
    pub prev: Token,  // previous token
    pub done: bool,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: lexer::Lexer<'a>) -> Self {
        Parser {
            lexer,
            token: Default::default(),
            prev: Default::default(),
            done: false,
        }
    }

    fn throw_exception_at(&self, error: ParseError, help: Option<&str>, token: Token) -> ! {
        let line = token.line;

        println!(
            "\x1b[31;1m{}:{}:{}:\x1b[0;1m {error}\x1b[0m",
            self.lexer.file.to_str().unwrap(),
            token.line,
            token.column
        );

        let lines = self.lexer.get_lines(line);
        if lines.len() == 0 {
            println!("No line information available :(");
            println!("This error shouldn't happen, please report it.");
            println!("Debug: token={}", token.clone())
        } else {
            // Determine the longest integer by digit so that we can make our error prettier.
            let longest_length = lines.iter().map(|it| it.0).max().unwrap().to_string().len();

            for (line_index, line_str) in lines {
                // We replace `\t` with ` ` so that no matter the terminal indentation, the underline will be aligned
                let indents = line_str.chars().filter(|it| *it == '\t').count();
                let sanitized = line_str.replace('\t', "    ");

                println!("\x1b[1;34m{line_index:>longest_length$} | \x1b[0m{sanitized}");
                if line_index == token.line {
                    // Determine the column that the token is on to highlight it
                    println!(
                        "\x1b[1;34m{} | {}\x1b[31m{}\x1b[0m",
                        " ".repeat(longest_length),
                        " ".repeat(token.column - 1 - indents + (indents * 4)),
                        "~".repeat(token.len)
                    );
                }
            }
        }

        if let Some(help) = help {
            println!("\x1b[1;32mhelp:\x1b[0m {}", help);
        }

        exit(1)
    }

    fn throw_exception(&self, error: ParseError, help: Option<&str>) -> ! {
        self.throw_exception_at(error, help, self.token.clone())
    }

    fn throw_exception_at_prev(&self, error: ParseError, help: Option<&str>) -> ! {
        self.throw_exception_at(error, help, self.prev.clone())
    }

    // #region: Token Utilities

    pub fn advance(&mut self) -> bool {
        self.prev = self.token.clone();

        let opt_tok = self.lexer.next_token();
        if opt_tok.is_none() {
            self.done = true;
            return false;
        }
        let tok = opt_tok.unwrap();
        if tok.is_err() {
            self.throw_exception(ParseError::AdvanceError(Box::new(tok.err().unwrap())), None);
        }

        self.token = tok.unwrap();
        return true;
    }

    pub fn accept(&mut self, kind: TokenKind) -> bool {
        if self.token.kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn expect(&mut self, kind: TokenKind, msg: &str) {
        if self.token.kind == kind {
            self.advance();
        } else {
            self.throw_exception(ParseError::ExpectedToken(self.token.clone()), Some(msg));
        }
    }

    // #endregion: Token Utilities

    // #region: Misc Parsing

    pub fn parse_type(&mut self) -> Node {
        let line = self.token.line;
        let column = self.token.column;

        let mut pointers = 0;
        while self.accept(TokenKind::Pointer) {
            pointers += 1
        }

        let name: String;
        let mut funptr_args: Option<Vec<Node>> = None;
        let mut funptr_rets: Option<Box<Node>> = None;

        if self.accept(TokenKind::KwFun) {
            name = "fun".to_string();
            let mut funptr_args_vec: Vec<Node> = vec![];
            self.expect(
                TokenKind::OpenParen,
                "expected open parenthesis after `fun` type",
            );
            if self.token.kind != TokenKind::CloseParen {
                loop {
                    let it = self.parse_type();
                    funptr_args_vec.push(it);
                    if !self.accept(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(
                TokenKind::CloseParen,
                "expected closing parenthesis for funptr",
            );
            funptr_args = Some(funptr_args_vec);

            if self.accept(TokenKind::Colon) {
                funptr_rets = Some(Box::new(self.parse_type().clone()));
            } else {
                funptr_rets = Some(Box::new(Node::get_void_type(line, column)));
            }
        } else {
            self.expect(TokenKind::Identifier, "expected type identifier");
            name = self.prev.text.clone();
        }

        let mut arrays: Vec<(Option<usize>, Option<String>)> = vec![];
        while self.accept(TokenKind::OpenBracket) {
            if funptr_rets.is_some() {
                self.throw_exception(ParseError::FunPtrWithArrays, None);
            }

            // `int[5]`
            if self.accept(TokenKind::Int) {
                arrays.push((Some(self.prev.text.parse::<usize>().unwrap()), None))
            }
            // `int[size]`
            else if self.accept(TokenKind::Identifier) {
                arrays.push((None, Some(self.prev.text.clone())))
            }
            // `int[]`
            else if self.token.kind == TokenKind::CloseBracket {
                arrays.push((None, None));
            }

            self.expect(
                TokenKind::CloseBracket,
                "expected closed bracket to match open bracket",
            );
        }

        Node {
            line,
            column,
            node: NodeKind::Type {
                pointers,
                name,
                arrays,
                funptr_args,
                funptr_rets,
            },
        }
    }

    pub fn parse_hashtags(&mut self) -> Vec<String> {
        if self.accept(TokenKind::OpenParen) {
            let mut tags: Vec<String> = vec![];
            loop {
                self.accept(TokenKind::Identifier);
                tags.push(self.prev.text.clone());
                if self.accept(TokenKind::CloseParen) {
                    break;
                }
                self.expect(
                    TokenKind::Comma,
                    "expected comma or closed parenthesis after identifier in hashtag list",
                )
            }
            tags
        } else {
            self.expect(
                TokenKind::Identifier,
                "expected identifier or parenthesis after hashtag (#)",
            );
            vec![self.prev.text.clone()]
        }
    }

    pub fn cast_hashtags<T: FromStr>(tags: Vec<String>) -> Vec<T>
    where
        <T as FromStr>::Err: Debug,
    {
        let mut casted: Vec<T> = vec![];
        for tag in tags {
            casted.push(T::from_str(tag.as_str()).expect("unknown hashtag"));
        }
        casted
    }

    pub fn parse_mac_invoke(&mut self) -> Node {
        let line = self.prev.line;
        let column = self.prev.column;

        self.expect(
            TokenKind::Identifier,
            "expected identifier after `@` to invoke a macro",
        );
        let name = self.prev.text.clone();
        self.expect(
            TokenKind::OpenParen,
            "expected open parenthesis, macro syntax: `'@' <id> '(' <params> ')'`",
        );

        if self.accept(TokenKind::CloseParen) {
            Node {
                line,
                column,
                node: NodeKind::ExprMacInvoke {
                    name,
                    params: vec![],
                },
            }
        } else {
            let mut params: Vec<Node> = vec![];
            loop {
                params.push(self.parse_expression());

                if !self.accept(TokenKind::Comma) {
                    break;
                }
            }
            self.expect(
                TokenKind::CloseParen,
                "expected closed parenthesis, macro syntax: `'@' <id> '(' <params> ')'`",
            );

            Node {
                line,
                column,
                node: NodeKind::ExprMacInvoke { name, params },
            }
        }
    }

    // #endregion: Misc Parsing

    // #region: Expressions

    pub fn parse_block(&mut self, advance: bool) -> Node {
        if advance {
            self.advance();
        }

        let line = self.prev.line;
        let column = self.prev.column;

        if self.prev.kind == TokenKind::OpenCurly {
            if self.accept(TokenKind::CloseCurly) {
                return Node {
                    line,
                    column,
                    node: NodeKind::ExprBlock(vec![]),
                };
            }

            let mut exprs: Vec<Node> = vec![];
            loop {
                exprs.push(self.parse_statement());
                if self.accept(TokenKind::CloseCurly) {
                    break;
                }
            }
            Node {
                line,
                column,
                node: NodeKind::ExprBlock(exprs),
            }
        } else if self.prev.kind == TokenKind::Arrow {
            Node {
                line,
                column,
                node: NodeKind::ExprBlock(vec![self.parse_statement()]),
            }
        } else {
            self.throw_exception_at_prev(ParseError::UnexpectedToken(self.prev.clone()), None)
        }
    }

    pub fn parse_let(&mut self) -> Node {
        let line = self.prev.line;
        let column = self.prev.column;

        self.expect(TokenKind::Identifier, "expected identifier after `let`");

        let name = self.prev.text.clone();

        let typ: Option<Box<Node>> = if self.accept(TokenKind::Colon) {
            Some(Box::new(self.parse_type()))
        } else {
            None
        };

        self.expect(TokenKind::Eq, "expected `=` after `let <id>`");

        let value = Box::new(self.parse_expression());

        return Node {
            line,
            column,
            node: NodeKind::ExprLet { name, typ, value },
        };
    }

    pub fn parse_var(&mut self) -> Node {
        let line = self.prev.line;
        let column = self.prev.column;

        self.expect(TokenKind::Identifier, "expected identifier after `var`");

        let name = self.prev.text.clone();

        let typ: Option<Box<Node>> = if self.accept(TokenKind::Colon) {
            Some(Box::new(self.parse_type()))
        } else {
            None
        };

        self.expect(TokenKind::Eq, "expected `=` after `var <id>`");

        let value = Box::new(self.parse_expression());

        return Node {
            line,
            column,
            node: NodeKind::ExprVar { name, typ, value },
        };
    }

    // Parses *non operator* expressions.
    pub fn parse_atom(&mut self) -> Node {
        let line = self.token.line;
        let column = self.token.column;

        let n = |node| Node { line, column, node };

        let atom = match *self {
            _ if self.accept(TokenKind::OpenParen) => {
                let node = n(NodeKind::ExprGroup(Box::new(self.parse_expression())));
                self.expect(
                    TokenKind::CloseParen,
                    "expected closed parenthesis to match open parenthesis in expression group",
                );
                node
            }

            // Literals
            _ if self.accept(TokenKind::Int) => n(NodeKind::ExprNumber(self.prev.text.clone())),
            _ if self.accept(TokenKind::Float) => n(NodeKind::ExprNumber(self.prev.text.clone())),
            _ if self.accept(TokenKind::Hex) => n(NodeKind::ExprNumber(self.prev.text.clone())),
            _ if self.accept(TokenKind::Binary) => n(NodeKind::ExprNumber(self.prev.text.clone())),
            _ if self.accept(TokenKind::String) => n(NodeKind::ExprString(self.prev.text.clone())),
            _ if self.accept(TokenKind::CString) => {
                n(NodeKind::ExprCString(self.prev.text.clone()))
            }
            _ if self.accept(TokenKind::Character) => n(NodeKind::ExprChar(self.prev.text.clone())),
            _ if self.accept(TokenKind::True) => n(NodeKind::ExprTrue),
            _ if self.accept(TokenKind::False) => n(NodeKind::ExprFalse),
            _ if self.accept(TokenKind::Identifier) => {
                n(NodeKind::ExprIdentifier(self.prev.text.clone()))
            }

            // List
            _ if self.accept(TokenKind::OpenBracket) => {
                if self.accept(TokenKind::CloseBracket) {
                    n(NodeKind::ExprList(vec![]))
                } else {
                    let mut nodes: Vec<Node> = vec![];

                    loop {
                        nodes.push(self.parse_expression());

                        if !self.accept(TokenKind::Comma) {
                            break;
                        }
                    }

                    self.expect(
                        TokenKind::CloseBracket,
                        "expected closed bracket (`]`) to end list expression.",
                    );

                    n(NodeKind::ExprList(nodes))
                }
            }

            // Blocks
            _ if self.accept(TokenKind::OpenCurly) => self.parse_block(false),
            _ if self.accept(TokenKind::Arrow) => self.parse_block(false),

            // Misc
            _ if self.accept(TokenKind::KwNew) => {
                self.expect(TokenKind::Identifier, "expected identifier after `new`");
                let id = self.prev.text.clone();
                let mut params: Vec<Node> = vec![];

                self.expect(
                    TokenKind::OpenParen,
                    "expected open parenthesis in `new` expression",
                );
                if self.token.kind != TokenKind::CloseParen {
                    loop {
                        params.push(self.parse_expression());
                        if !self.accept(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(
                    TokenKind::CloseParen,
                    "expected closed parenthesis to end `new` expression",
                );

                n(NodeKind::ExprNew { id, params })
            }

            // Prefix unary operators
            _ if self.accept(TokenKind::OpNot) => n(NodeKind::ExprUnaryOperator {
                kind: OperatorKind::Not,
                value: Box::new(self.parse_atom()),
            }),
            _ if self.accept(TokenKind::OpSub) => n(NodeKind::ExprUnaryOperator {
                kind: OperatorKind::Negate,
                value: Box::new(self.parse_atom()),
            }),
            _ if self.accept(TokenKind::KwRef) => n(NodeKind::ExprUnaryOperator {
                kind: OperatorKind::Ref,
                value: Box::new(self.parse_atom()),
            }),

            _ => self.throw_exception(ParseError::ExpectedExpression(self.token.clone()), None),
        };

        self.parse_postfix(atom)
    }

    pub fn parse_postfix(&mut self, node: Node) -> Node {
        let line = node.line;
        let column = node.column;
        let n = |node| Node { line, column, node };

        let mut atom = node;

        // Postfix operators
        loop {
            if self.accept(TokenKind::Pointer) {
                atom = n(NodeKind::ExprUnaryOperator {
                    kind: OperatorKind::Deref,
                    value: Box::new(atom),
                })
            } else if self.accept(TokenKind::OpenParen) {
                if self.accept(TokenKind::CloseParen) {
                    atom = n(NodeKind::ExprInvoke {
                        left: Box::new(atom),
                        params: vec![],
                    })
                } else {
                    let mut params: Vec<Node> = vec![];
                    loop {
                        params.push(self.parse_expression());

                        if !self.accept(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(
                        TokenKind::CloseParen,
                        "expected closed parenthesis to end parameter list",
                    );
                    atom = n(NodeKind::ExprInvoke {
                        left: Box::new(atom),
                        params,
                    })
                }
            } else if self.accept(TokenKind::OpenBracket) {
                let right = self.parse_expression();
                self.expect(
                    TokenKind::CloseBracket,
                    "expected closed bracket (`]`) to end index operator",
                );
                atom = n(NodeKind::ExprBinaryOperator {
                    kind: OperatorKind::Index,
                    left: Box::new(atom),
                    right: Box::new(right),
                });
            } else if self.accept(TokenKind::KwAs) {
                let typ = self.parse_type();
                atom = n(NodeKind::ExprBinaryOperator {
                    kind: OperatorKind::As,
                    left: Box::new(atom),
                    right: Box::new(typ),
                })
            } else {
                break;
            }
        }

        atom
    }

    pub fn parse_expression_inner(&mut self, left_node: Node, min_prec: u8) -> Node {
        // https://en.wikipedia.org/wiki/Operator-precedence_parser#Pseudocode
        let mut left_atom = left_node.clone();
        let mut lookahead = self.token.clone();
        let mut lookahead_op = lookahead.kind.get_operator().expect("expected operator");

        while lookahead.kind.is_operator() && lookahead_op.prec >= min_prec {
            let op = lookahead_op;
            self.advance();
            // let mut right_atom = self.parse_atom();
            let mut right_atom = if op.kind == OperatorKind::As {
                self.parse_type()
            } else {
                self.parse_atom()
            };

            lookahead = self.token.clone();
            if lookahead.kind.is_operator() {
                lookahead_op = lookahead.kind.get_operator().expect("expected operator");
                while lookahead.kind.is_operator()
                    && (lookahead_op.prec > op.prec
                        || (lookahead_op.assoc == Associativity::Right
                            && lookahead_op.prec == op.prec))
                {
                    if lookahead_op.kind == OperatorKind::As {
                        right_atom = self.parse_type();
                    } else {
                        right_atom = self.parse_expression_inner(
                            right_atom,
                            op.prec + if lookahead_op.prec > op.prec { 1 } else { 0 },
                        );
                    }
                    lookahead = self.token.clone();
                    if lookahead.kind.is_operator() {
                        lookahead_op = lookahead.kind.get_operator().expect("expected operator");
                    } else {
                        break;
                    }
                }
            }

            left_atom = Node::join(op.kind, left_atom, right_atom);
        }

        self.parse_postfix(left_atom)
    }

    pub fn parse_expression(&mut self) -> Node {
        if self.accept(TokenKind::KwLet) {
            self.parse_let()
        } else if self.accept(TokenKind::KwVar) {
            self.parse_var()
        } else {
            let left = self.parse_atom();
            let it = if self.token.kind.is_operator() {
                self.parse_expression_inner(left, 0)
            } else {
                left
            };
            return self.parse_postfix(it);
        }
    }

    // #endregion: Expressions

    // #region: Statements

    pub fn parse_ret(&mut self) -> Node {
        Node {
            line: self.prev.line,
            column: self.prev.column,
            node: NodeKind::StatRet(Some(Box::new(self.parse_expression()))),
        }
    }

    pub fn parse_if(&mut self) -> Node {
        let line = self.token.line;
        let column = self.token.column;

        let cond = self.parse_expression();
        let expr = self.parse_block(true);
        let mut else_: Option<Box<Node>> = None;
        if self.accept(TokenKind::KwElse) {
            // the only statement allowed after `else` is another `if` statement, otherwise it must be a block
            if self.accept(TokenKind::KwIf) {
                else_ = Some(Box::new(self.parse_if()));
            } else {
                else_ = Some(Box::new(self.parse_block(true)));
            }
        }

        Node {
            line,
            column,
            node: NodeKind::StatIf {
                cond: Box::new(cond),
                expr: Box::new(expr),
                else_,
            },
        }
    }

    pub fn parse_switch(&mut self) -> Node {
        let line = self.token.line;
        let column = self.token.column;

        let switch = Box::new(self.parse_expression());
        self.expect(
            TokenKind::OpenCurly,
            "expected open curly brace (`{`) after `switch <expression>`",
        );

        let mut cases: Vec<(Option<Box<Node>>, bool, Box<Node>)> = vec![];
        loop {
            let is_fall_case = self.accept(TokenKind::KwFall);

            if self.accept(TokenKind::KwElse) {
                if is_fall_case {
                    self.throw_exception(
                        ParseError::UnexpectedToken(self.prev.clone()),
                        Some("`fall` cannot be used with `else`"),
                    )
                }

                let block = self.parse_block(true);
                cases.push((None, is_fall_case, Box::new(block)));

                if !self.accept(TokenKind::CloseCurly) {
                    self.throw_exception(
                        ParseError::UnexpectedToken(self.prev.clone()),
                        Some("`else` must be the last case of a `switch` statement."),
                    )
                }

                break;
            } else {
                self.expect(TokenKind::KwCase, "expected `case`");
                let case = Some(Box::new(self.parse_expression()));
                let block = self.parse_block(true);
                cases.push((case, is_fall_case, Box::new(block)));

                if self.accept(TokenKind::CloseCurly) {
                    break;
                }
            }
        }

        Node {
            line,
            column,
            node: NodeKind::StatSwitch { switch, cases },
        }
    }

    pub fn parse_for(&mut self) -> Node {
        let line = self.token.line;
        let column = self.token.column;

        let leftmost_expr = self.parse_expression();

        // c style for loop
        if self.accept(TokenKind::Semicolon) {
            let cond = self.parse_expression();
            self.expect(TokenKind::Semicolon, "C-style for loops require three expressions, separated by semicolons (i.e, `for <expr> ; <expr> ; <expr>`)");
            let inc = self.parse_expression();
            let expr = self.parse_block(true);

            Node {
                line,
                column,
                node: NodeKind::StatForCStyle {
                    def: Box::new(leftmost_expr),
                    cond: Box::new(cond),
                    inc: Box::new(inc),
                    expr: Box::new(expr),
                },
            }
        }
        // single expr for loop
        else if self.accept(TokenKind::OpenCurly) || self.accept(TokenKind::Arrow) {
            let expr = self.parse_block(false);
            Node {
                line,
                column,
                node: NodeKind::StatForSingleExpr {
                    cond: Box::new(leftmost_expr),
                    expr: Box::new(expr),
                },
            }
        }
        // range for loop
        else {
            // we get this now because if we accept a KwIn then we wouldn't be able to access this ID afterwards
            let potentially_id = self.prev.text.clone();

            // for <id> in <expr> to <expr>
            if self.accept(TokenKind::KwIn) {
                let from = self.parse_expression();
                self.expect(
                    TokenKind::KwTo,
                    "Range for loop syntax: `for (<id> in)? <expr> to <expr>`",
                );
                let to = self.parse_expression();
                let expr = self.parse_block(true);

                Node {
                    line,
                    column,
                    node: NodeKind::StatForRange {
                        var: Some(potentially_id),
                        from: Box::new(from),
                        to: Box::new(to),
                        expr: Box::new(expr),
                    },
                }
            }
            // for <expr> to <expr>
            else {
                self.expect(
                    TokenKind::KwTo,
                    "Range for loop syntax: `for (<id> in)? <expr> to <expr>`",
                );
                let to = self.parse_atom();
                let expr = self.parse_block(true);

                Node {
                    line,
                    column,
                    node: NodeKind::StatForRange {
                        var: None,
                        from: Box::new(leftmost_expr),
                        to: Box::new(to),
                        expr: Box::new(expr),
                    },
                }
            }
        }
    }

    fn parse_raw(&mut self) -> Node {
        let line = self.token.line;
        let column = self.token.column;

        // We don't need to check for brackets here since the lexer does that for us
        self.expect(
            TokenKind::LiteralText,
            "internal error, literal text within raw[] block not found.",
        );
        let text = self.prev.text.clone();

        Node {
            line,
            column,
            node: NodeKind::Raw(text),
        }
    }

    pub fn parse_statement(&mut self) -> Node {
        match *self {
            _ if self.accept(TokenKind::KwRet) => self.parse_ret(),
            _ if self.accept(TokenKind::KwIf) => self.parse_if(),
            _ if self.accept(TokenKind::KwSwitch) => self.parse_switch(),
            _ if self.accept(TokenKind::KwFor) => self.parse_for(),
            _ if self.accept(TokenKind::KwRaw) => self.parse_raw(),
            _ if self.accept(TokenKind::At) => self.parse_mac_invoke(),
            // if nothing works, we'll try to parse an expression, and if *that* doesn't work, then we have a syntax error
            _ => {
                let expr = self.parse_expression();
                Node {
                    line: expr.line,
                    column: expr.column,
                    node: NodeKind::StatExpr(Box::new(expr)),
                }
            }
        }
    }

    // #endregion: Statements

    // #region: Top level statements

    pub fn parse_use(&mut self) -> Node {
        let line = self.prev.line;
        let column = self.prev.column;

        let mut path = PathBuf::new();
        self.expect(TokenKind::Identifier, "expected identifier after `use`");
        path.push(self.prev.text.clone());
        while self.accept(TokenKind::OpDiv) {
            self.expect(
                TokenKind::Identifier,
                "expected identifier after `/` in `use`",
            );
            path.push(self.prev.text.clone());
        }

        let selections = if self.accept(TokenKind::OpenBracket) {
            let mut s: Vec<String> = vec![];
            loop {
                self.expect(
                    TokenKind::Identifier,
                    "expected identifier in selective `use` statement",
                );

                s.push(self.prev.text.clone());

                if !self.accept(TokenKind::Comma) {
                    break;
                }
            }
            self.expect(
                TokenKind::CloseBracket,
                "expected closed bracket (`]`) to end selective `use` statement",
            );
            Some(s)
        } else {
            None
        };

        Node {
            line,
            column,
            node: NodeKind::TopUse(path, selections),
        }
    }

    pub fn parse_fun(&mut self, tags: Vec<FunTags>) -> Node {
        let line = self.prev.line;
        let column = self.prev.column;

        self.expect(TokenKind::Identifier, "expected identifier after `fun`");
        let id = self.prev.text.clone();
        let mut params: Vec<(String, Node)> = vec![];

        self.expect(
            TokenKind::OpenParen,
            "expected open parenthesis after function name",
        );
        while self.accept(TokenKind::Identifier) {
            let param_id = self.prev.text.clone();
            self.expect(
                TokenKind::Colon,
                "expected colon in between parameter ID and its type",
            );
            let typ = self.parse_type();
            params.push((param_id, typ));
            if !self.accept(TokenKind::Comma) {
                // if there is no comma then we must be on the last parameter
                break;
            }
        }
        self.expect(
            TokenKind::CloseParen,
            "expected closed parenthesis after function parameter list",
        );

        let rets: Box<Node> = if self.accept(TokenKind::Colon) {
            Box::new(self.parse_type())
        } else {
            Box::new(Node::get_void_type(line, column))
        };

        let expr = Box::new(self.parse_block(true));

        Node {
            line,
            column,
            node: NodeKind::TopFun {
                tags,
                id,
                params,
                rets,
                expr,
            },
        }
    }

    pub fn parse_rec(&mut self, tags: Vec<RecTags>) -> Node {
        let line = self.prev.line;
        let column = self.prev.column;

        self.expect(TokenKind::Identifier, "expected identifier after `rec`");
        let id = self.prev.text.clone();
        let mut fields: Vec<(String, Node)> = vec![];

        self.expect(
            TokenKind::OpenParen,
            "expected open parenthesis after record name",
        );
        while self.accept(TokenKind::Identifier) {
            let param_id = self.prev.text.clone();
            self.expect(
                TokenKind::Colon,
                "expected colon in between field ID and its type",
            );
            let typ = self.parse_type();
            fields.push((param_id, typ));
            if !self.accept(TokenKind::Comma) {
                // if there is no comma then we must be on the last field
                break;
            }
        }
        self.expect(
            TokenKind::CloseParen,
            "expected closed parenthesis after record field list",
        );

        Node {
            line,
            column,
            node: NodeKind::TopRec { tags, id, fields },
        }
    }

    fn parse_def(&mut self, tags: Vec<DefTags>) -> Node {
        let line = self.prev.line;
        let column = self.prev.column;

        self.expect(TokenKind::Identifier, "expected identifier after `def`");
        let id = self.prev.text.clone();
        self.expect(TokenKind::Eq, "expected `=` after `def <identifier>`");
        let typ = self.parse_type();

        Node {
            line,
            column,
            node: NodeKind::TopDef {
                tags,
                id,
                typ: Box::new(typ),
            },
        }
    }

    fn parse_tag(&mut self, tags: Vec<TagTags>) -> Node {
        let line = self.prev.line;
        let column = self.prev.column;

        self.expect(TokenKind::Identifier, "expected identifier after `tag`");
        let id = self.prev.text.clone();
        let mut entries: Vec<(String, Option<Box<Node>>)> = vec![];

        self.expect(
            TokenKind::OpenParen,
            "expected open parenthesis after tag name",
        );
        while self.accept(TokenKind::Identifier) {
            let entry_id = self.prev.text.clone();
            if self.accept(TokenKind::Eq) {
                let atom = self.parse_atom();
                match atom.node {
                    NodeKind::ExprNumber(_) => entries.push((entry_id, Some(Box::new(atom)))),
                    _ => {
                        self.throw_exception(ParseError::ExpectedTokenOfKind(TokenKind::Int), None)
                    }
                }
            } else {
                entries.push((entry_id, None))
            }
            self.accept(TokenKind::Comma); // commas are optional for enums
        }
        self.expect(
            TokenKind::CloseParen,
            "expected closed parenthesis after tag entry list",
        );

        Node {
            line,
            column,
            node: NodeKind::TopTag { tags, id, entries },
        }
    }

    fn parse_tagrec(&mut self, tags: Vec<TagRecTags>) -> Node {
        let line = self.prev.line;
        let column = self.prev.column;

        self.expect(TokenKind::Identifier, "expected identifier after `tag`");
        let id = self.prev.text.clone();
        let mut entries: Vec<(String, Vec<(String, Node)>)> = vec![];

        self.expect(
            TokenKind::OpenParen,
            "expected open parenthesis after tag name",
        );
        while self.accept(TokenKind::Identifier) {
            let entry_id = self.prev.text.clone();
            let mut entry_entries: Vec<(String, Node)> = vec![];

            // tagrec entry parameters
            if self.accept(TokenKind::OpenParen) {
                while self.accept(TokenKind::Identifier) {
                    let param_id = self.prev.text.clone();
                    self.expect(
                        TokenKind::Colon,
                        "expected colon in between field ID and its type",
                    );
                    let typ = self.parse_type();
                    entry_entries.push((param_id, typ));
                    if !self.accept(TokenKind::Comma) {
                        // if there is no comma then we must be on the last field
                        break;
                    }
                }
                self.expect(
                    TokenKind::CloseParen,
                    "expected closed parenthesis after `tag rec` entry list",
                )
            }

            entries.push((entry_id, entry_entries));
            self.accept(TokenKind::Comma); // commas are optional for enums
        }
        self.expect(
            TokenKind::CloseParen,
            "expected closed parenthesis after tag entry list",
        );

        Node {
            line,
            column,
            node: NodeKind::TopTagRec { tags, id, entries },
        }
    }

    pub fn parse_pragma(&mut self) -> Node {
        let line = self.prev.line;
        let column = self.prev.column;

        self.expect(TokenKind::Identifier, "expected identifier after `pragma`");
        let id = self.prev.text.clone();

        let mut params: Vec<Node> = vec![];
        self.expect(
            TokenKind::OpenParen,
            "expected open parenthesis after pragma identifier",
        );
        if !self.accept(TokenKind::CloseParen) {
            loop {
                params.push(self.parse_expression());

                if !self.accept(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(
            TokenKind::CloseParen,
            "expected closed parenthesis after pragma argument list",
        );

        Node {
            line,
            column,
            node: NodeKind::TopPragma { id, params },
        }
    }

    pub fn parse_top_level_statement(&mut self) -> Node {
        if self.accept(TokenKind::KwUse) {
            self.parse_use()
        } else if self.accept(TokenKind::At) {
            self.parse_mac_invoke()
        } else if self.accept(TokenKind::Hashtag) {
            let tags = self.parse_hashtags();
            if self.accept(TokenKind::KwFun) {
                self.parse_fun(Parser::cast_hashtags::<FunTags>(tags))
            } else if self.accept(TokenKind::KwRec) {
                self.parse_rec(Parser::cast_hashtags::<RecTags>(tags))
            } else if self.accept(TokenKind::KwDef) {
                self.parse_def(Parser::cast_hashtags::<DefTags>(tags))
            } else if self.accept(TokenKind::KwTag) {
                if self.accept(TokenKind::KwRec) {
                    self.parse_tagrec(Parser::cast_hashtags::<TagRecTags>(tags))
                } else {
                    self.parse_tag(Parser::cast_hashtags::<TagTags>(tags))
                }
            } else {
                self.throw_exception(
                    ParseError::UnexpectedToken(self.token.clone()),
                    Some("hashtags can only be applied to [fun, rec, def, mac, tag, tag rec]"),
                )
            }
        } else if self.accept(TokenKind::KwFun) {
            self.parse_fun(vec![])
        } else if self.accept(TokenKind::KwRec) {
            self.parse_rec(vec![])
        } else if self.accept(TokenKind::KwDef) {
            self.parse_def(vec![])
        } else if self.accept(TokenKind::KwTag) {
            if self.accept(TokenKind::KwRec) {
                self.parse_tagrec(vec![])
            } else {
                self.parse_tag(vec![])
            }
        } else if self.accept(TokenKind::KwRaw) {
            self.parse_raw()
        } else if self.accept(TokenKind::KwPragma) {
            self.parse_pragma()
        } else if self.accept(TokenKind::KwVar) {
            let it = self.parse_var();
            Node {
                line: it.line,
                column: it.line,
                node: NodeKind::StatExpr(Box::new(it)),
            }
        } else if self.accept(TokenKind::KwLet) {
            let it = self.parse_let();
            Node {
                line: it.line,
                column: it.line,
                node: NodeKind::StatExpr(Box::new(it)),
            }
        } else {
            self.throw_exception(ParseError::UnexpectedToken(self.token.clone()), None);
        }
    }

    // #endregion: Top level statements

    pub fn parse(&mut self, add_implicit_use_std: bool) -> Node {
        let mut nodes: Vec<Node> = vec![];
        if add_implicit_use_std {
            nodes.push(Node {
                line: 0,
                column: 0,
                node: NodeKind::TopUse(PathBuf::from("std"), None),
            })
        }
        self.advance();
        while !self.done {
            nodes.push(self.parse_top_level_statement());
        }
        Node {
            line: 0,
            column: 0,
            node: NodeKind::Program(nodes),
        }
    }
}

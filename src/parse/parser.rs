use std::{collections::HashMap, fmt::Debug, path::PathBuf, str::FromStr};

use crate::{hashtags::FunTags, parse::operator::Associativity};

use super::{
    ast::Node,
    error::ParseError,
    lexer,
    token::{Token, TokenKind},
};

pub struct Parser<'a> {
    pub lexer: lexer::Lexer<'a>,
    pub token: Token, // current token
    pub prev: Token,  // previous token
    pub done: bool,
}

impl<'a> Parser<'a> {
    pub fn advance(&mut self) -> bool {
        self.prev = self.token.clone();

        let opt_tok = self.lexer.next_token();
        if opt_tok.is_none() {
            self.done = true;
            return false;
        }
        let tok = opt_tok.unwrap();
        if tok.is_err() {
            panic!("parsing advance error: {}", tok.err().unwrap());
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
            panic!("parsing error: {}", msg);
        }
    }

    pub fn parse_hashtags<T: FromStr>(&mut self) -> Vec<T>
    where
        <T as FromStr>::Err: Debug,
    {
        if self.accept(TokenKind::OpenParen) {
            let mut tags: Vec<T> = vec![];
            loop {
                self.accept(TokenKind::Identifier);
                tags.push(T::from_str(self.prev.text.as_str()).expect("unknown hashtag"));
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
            vec![T::from_str(self.prev.text.as_str()).expect("unknown hashtag")]
        }
    }

    // Parses *non operator* expressions.
    pub fn parse_atom(&mut self) -> Result<Node, ParseError> {
        // println!("atom: {}", self.token);
        match *self {
            _ if self.accept(TokenKind::OpenParen) => {
                let node = Node::ExprGroup(Box::new(
                    self.parse_expression()
                        .expect("expected expression inside group"),
                ));
                self.expect(
                    TokenKind::CloseParen,
                    "expected closed parenthesis to match open parenthesis in expression group",
                );
                Ok(node)
            }

            // Literals
            _ if self.accept(TokenKind::Int) => Ok(Node::ExprNumber(self.prev.text.clone())),
            _ if self.accept(TokenKind::Float) => Ok(Node::ExprNumber(self.prev.text.clone())),
            _ if self.accept(TokenKind::Hex) => Ok(Node::ExprNumber(self.prev.text.clone())),
            _ if self.accept(TokenKind::Binary) => Ok(Node::ExprNumber(self.prev.text.clone())),
            _ if self.accept(TokenKind::String) => Ok(Node::ExprString(self.prev.text.clone())),
            _ if self.accept(TokenKind::Character) => Ok(Node::ExprChar(
                self.prev.text.clone().chars().nth(1).unwrap(),
            )),
            _ if self.accept(TokenKind::True) => Ok(Node::ExprTrue),
            _ if self.accept(TokenKind::False) => Ok(Node::ExprFalse),
            _ if self.accept(TokenKind::Identifier) => {
                Ok(Node::ExprIdentifier(self.prev.text.clone()))
            }

            // Blocks
            _ if self.accept(TokenKind::OpenCurly) => {
                if self.accept(TokenKind::CloseCurly) {
                    return Ok(Node::ExprBlock(vec![]));
                }

                let mut exprs: Vec<Node> = vec![];
                loop {
                    exprs.push(self.parse_statement()?);
                    if self.accept(TokenKind::CloseCurly) {
                        break;
                    }
                }
                Ok(Node::ExprBlock(exprs))
            }
            _ if self.accept(TokenKind::Arrow) => {
                Ok(Node::ExprBlock(vec![self.parse_statement()?]))
            }

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
                        params.push(self.parse_atom()?);
                        if !self.accept(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(
                    TokenKind::CloseParen,
                    "expected closed parenthesis to end `new` expression",
                );

                Ok(Node::ExprNew { id, params })
            }

            _ => Err(ParseError::ExpectedExpression),
        }
    }

    pub fn parse_expression_inner(&mut self, left_node: Node, min_prec: u8) -> Node {
        // println!("pei:");
        // https://en.wikipedia.org/wiki/Operator-precedence_parser#Pseudocode
        let mut left_atom = left_node.clone();
        let mut lookahead = self.token.clone();
        let mut lookahead_op = lookahead.kind.get_operator().expect("expected operator");
        while lookahead.kind.is_operator() && lookahead_op.prec >= min_prec {
            let op = lookahead_op;
            self.advance();
            let mut right_atom = self.parse_atom().unwrap();
            // println!("  right atom: {}", right_atom);
            lookahead = self.token.clone();
            // println!("  lookahead: {}", lookahead);
            // println!("  isop: {}", lookahead.kind.is_operator());
            if lookahead.kind.is_operator() {
                lookahead_op = lookahead.kind.get_operator().expect("expected operator");
                while lookahead.kind.is_operator()
                    && (lookahead_op.prec > op.prec
                        || (lookahead_op.assoc == Associativity::Right
                            && lookahead_op.prec == op.prec))
                {
                    right_atom = self.parse_expression_inner(
                        right_atom,
                        op.prec + if lookahead_op.prec > op.prec { 1 } else { 0 },
                    );
                    lookahead = self.token.clone();
                    if lookahead.kind.is_operator() {
                        lookahead_op = lookahead.kind.get_operator().expect("expected operator");
                    } else {
                        break;
                    }
                }
            }
            left_atom = Node::join(op.kind, left_atom, right_atom);
            // println!("  joined: {}", left_atom);
            // println!("  lookahead: {}", lookahead);
            // println!("  precs: {} > {}", lookahead_op.prec, min_prec);
        }
        // println!("end pei: {}", left_atom);
        left_atom
    }

    pub fn parse_expression(&mut self) -> Result<Node, ParseError> {
        let left = self.parse_atom()?;
        if self.token.kind.is_operator() {
            Ok(self.parse_expression_inner(left, 0))
        } else {
            Ok(left)
        }
    }

    pub fn parse_statement(&mut self) -> Result<Node, ParseError> {
        // println!("stat: {}", self.token);
        match *self {
            _ if self.accept(TokenKind::KwRet) => {
                Ok(Node::StatRet(Some(Box::new(self.parse_expression()?))))
            }
            _ => Err(ParseError::ExpectedStatement),
        }
    }

    pub fn parse_type(&mut self) -> Result<Node, ParseError> {
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
                    let it = self.parse_type()?;
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
                funptr_rets = Some(Box::new(self.parse_type()?.clone()));
            } else {
                funptr_rets = Some(Box::new(Node::get_void_type()));
            }
        } else {
            self.expect(TokenKind::Identifier, "expected type identifier");
            name = self.prev.text.clone();
        }

        let mut arrays: Vec<(Option<usize>, Option<String>)> = vec![];
        while self.accept(TokenKind::OpenBracket) {
            if funptr_rets.is_some() {
                return Err(ParseError::FunPtrWithArrays);
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

        Ok(Node::Type {
            pointers,
            name,
            arrays,
            funptr_args,
            funptr_rets,
        })
    }

    pub fn parse_fun(&mut self, tags: Vec<FunTags>) -> Result<Node, ParseError> {
        self.expect(TokenKind::Identifier, "expected identifier after `fun`");
        let id = self.prev.text.clone();
        let mut params: HashMap<String, Node> = HashMap::new();

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
            let typ = self.parse_type()?;
            params.insert(param_id, typ);
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
            Box::new(self.parse_type()?)
        } else {
            Box::new(Node::get_void_type())
        };

        let expr = Box::new(
            self.parse_atom()
                .expect("expected expression after function declaration"),
        );

        Ok(Node::TopFun {
            tags,
            id,
            params,
            rets,
            expr,
        })
    }

    pub fn parse_top_level_statement(&mut self) -> Result<Node, ParseError> {
        if self.accept(TokenKind::KwUse) {
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
            return Ok(Node::TopUse(path));
        } else if self.accept(TokenKind::Hashtag) {
            let hashtags = self.parse_hashtags::<FunTags>();
            if self.accept(TokenKind::KwFun) {
                return self.parse_fun(hashtags);
            }
        } else if self.accept(TokenKind::KwFun) {
            return self.parse_fun(vec![]);
        }
        return Err(ParseError::UnexpectedToken(self.token.clone()));
    }

    pub fn parse(&mut self) -> Result<Node, ParseError> {
        let mut nodes: Vec<Node> = vec![];
        self.advance();
        while !self.done {
            nodes.push(self.parse_top_level_statement()?);
        }
        Ok(Node::Program(nodes))
    }
}

pub fn make_parser<'a>(lexer: lexer::Lexer<'a>) -> Parser<'a> {
    Parser {
        lexer,
        token: Default::default(),
        prev: Default::default(),
        done: false,
    }
}

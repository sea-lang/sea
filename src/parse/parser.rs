use std::{collections::HashMap, fmt::Debug, path::PathBuf, str::FromStr};

use crate::hashtags::FunTags;

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
    fn advance(&mut self) -> bool {
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

    fn accept(&mut self, kind: TokenKind) -> bool {
        if self.token.kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind, msg: &str) {
        if self.token.kind == kind {
            self.advance();
        } else {
            panic!("parsing error: {}", msg);
        }
    }

    fn parse_hashtags<T: FromStr>(&mut self) -> Vec<T>
    where
        <T as FromStr>::Err: Debug,
    {
        if self.accept(TokenKind::OpenParen) {
            let mut tags: Vec<T> = vec![];
            loop {
                self.accept(TokenKind::Identifier);
                tags.push(T::from_str("static").expect("unknown hashtag"));
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

    fn parse_expression(&mut self) -> Result<Node, ParseError> {
        // println!("parsing expression: {}", self.token);
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
            _ if self.accept(TokenKind::OpenCurly) => {
                if self.accept(TokenKind::CloseCurly) {
                    return Ok(Node::ExprBlock(vec![]));
                }

                let mut exprs: Vec<Node> = vec![];
                loop {
                    exprs.push(self.parse_statement().unwrap());
                    if self.accept(TokenKind::CloseCurly) {
                        break;
                    }
                }
                Ok(Node::ExprBlock(exprs))
            }
            _ if self.accept(TokenKind::Arrow) => {
                Ok(Node::ExprBlock(vec![self.parse_statement().unwrap()]))
            }
            _ => Err(ParseError::ExpectedExpression),
        }
    }

    fn parse_statement(&mut self) -> Result<Node, ParseError> {
        // println!("stat: {}", self.token);
        match *self {
            _ if self.accept(TokenKind::KwRet) => Ok(Node::StatRet(Some(Box::new(
                self.parse_expression().unwrap(),
            )))),
            _ => Err(ParseError::ExpectedStatement),
        }
    }

    fn parse_type(&mut self) -> Result<Node, ParseError> {
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
                    let it = self.parse_type().unwrap();
                    println!("funptrarg: {}", it);
                    funptr_args_vec.push(it);
                    println!("tttttok: {}", self.token);
                    if !self.accept(TokenKind::Comma) {
                        println!("no comma!");
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
                funptr_rets = Some(Box::new(self.parse_type().unwrap().clone()));
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

    fn parse_fun(&mut self, tags: Vec<FunTags>) -> Result<Node, ParseError> {
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
            let typ = self.parse_type().unwrap();
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
            Box::new(self.parse_type().unwrap())
        } else {
            Box::new(Node::get_void_type())
        };

        let expr = Box::new(
            self.parse_expression()
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

    fn parse_top_level_statement(&mut self) -> Result<Node, ParseError> {
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
            nodes.push(self.parse_top_level_statement().unwrap());
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

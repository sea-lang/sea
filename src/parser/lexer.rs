use std::{collections::HashMap, iter::Peekable, str::Chars, sync::LazyLock};

use super::{
    error::ParseError,
    token::{Token, TokenKind},
};

pub struct Lexer<'a> {
    code: Peekable<Chars<'a>>, // the source code being lexed
    length: usize,             // the length of the source code
    start: usize,              // the index at which the current token started
    pos: usize,                // the index to the current character
    line: usize,               // the current line that the lexer is on
    cur: char,                 // the current character
    buffer: String,            // all characters since `start`
}

static KEYWORDS: LazyLock<HashMap<&str, TokenKind>> = LazyLock::new(|| {
    HashMap::from([
        ("use", TokenKind::KwUse),
        ("rec", TokenKind::KwRec),
        ("fun", TokenKind::KwFun),
        ("var", TokenKind::KwVar),
        ("let", TokenKind::KwLet),
        ("ret", TokenKind::KwRet),
        ("raw", TokenKind::KwRaw),
        ("if", TokenKind::KwIf),
        ("else", TokenKind::KwElse),
        ("for", TokenKind::KwFor),
        ("each", TokenKind::KwEach),
        ("of", TokenKind::KwOf),
        ("new", TokenKind::KwNew),
        ("ref", TokenKind::KwRef),
        ("as", TokenKind::KwAs),
        ("to", TokenKind::KwTo),
        ("in", TokenKind::KwIn),
        ("def", TokenKind::KwDef),
        ("mac", TokenKind::KwMac),
        ("lit", TokenKind::KwLit),
        ("tag", TokenKind::KwTag),
        ("switch", TokenKind::KwSwitch),
        ("case", TokenKind::KwCase),
        ("fall", TokenKind::KwFall),
        ("true", TokenKind::True),
        ("false", TokenKind::False),
    ])
});

fn is_valid_id_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || ch == '$'
}

fn is_valid_id(ch: char) -> bool {
    is_valid_id_start(ch) || ch.is_numeric()
}

impl<'a> Lexer<'a> {
    fn skip(&mut self) {
        self.skip_no_buffer();
        self.buffer.push(self.cur);
    }

    fn skip_no_buffer(&mut self) {
        self.cur = self.peek();
        self.code.next();
        self.pos += 1;
    }

    fn advance(&mut self) -> char {
        let it = self.peek();
        self.buffer.push(it);
        self.skip_no_buffer();
        it
    }

    fn peek(&mut self) -> char {
        self.code.peek().map_or('\0', |it| *it)
    }

    fn is_done(&self) -> bool {
        self.pos >= self.length
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            start: self.start,
            len: self.pos - self.start,
            text: self.buffer.clone(),
            line: self.line,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let ch = self.peek();
            match ch {
                '\n' => {
                    self.skip_no_buffer();
                    self.line += 1;
                }
                ch if ch.is_whitespace() => self.skip(),
                _ => return,
            }
        }
    }

    fn lex_string(&mut self) -> Result<Token, ParseError> {
        self.start += 1; // skip the opening quote
        self.buffer.remove(0); // remove the opening quote from the buffer

        while self.peek() != '"' && !self.is_done() {
            if self.cur == '\n' {
                self.line += 1;
            }
            self.skip();
        }

        if self.is_done() {
            Err(ParseError::UnterminatedString)
        } else {
            self.skip_no_buffer(); // eat the closing quote
            Ok(self.make_token(TokenKind::String))
        }
    }

    fn lex_number(&mut self) -> Result<Token, ParseError> {
        let mut is_float = false;

        while self.peek().is_alphanumeric() || self.peek() == '_' || self.peek() == '.' {
            if self.peek() == '.' {
                // prevent two dots in one float
                if is_float {
                    return Err(ParseError::UnexpectedCharacter('.'));
                }

                is_float = true;
            }

            self.skip();
        }

        Ok(self.make_token(if is_float {
            TokenKind::Float
        } else {
            TokenKind::Int
        }))
    }

    fn lex_id_or_keyword(&mut self) -> Result<Token, ParseError> {
        while is_valid_id(self.peek()) || self.is_done() {
            self.skip();
        }

        if KEYWORDS.contains_key(self.buffer.as_str()) {
            Ok(self.make_token(KEYWORDS[self.buffer.as_str()]))
        } else {
            Ok(self.make_token(TokenKind::Identifier))
        }
    }

    pub fn next_token(&mut self) -> Option<Result<Token, ParseError>> {
        self.skip_whitespace();

        self.buffer.clear();
        self.start = self.pos;

        let cur = self.advance();

        if self.is_done() {
            None
        } else {
            match cur {
                // Common/misc symbols
                ',' => Some(Ok(self.make_token(TokenKind::Comma))),
                ':' => Some(Ok(self.make_token(TokenKind::Colon))),
                ';' => Some(Ok(self.make_token(TokenKind::Semicolon))),
                '^' => Some(Ok(self.make_token(TokenKind::Pointer))),
                '(' => Some(Ok(self.make_token(TokenKind::OpenParen))),
                ')' => Some(Ok(self.make_token(TokenKind::CloseParen))),
                '[' => Some(Ok(self.make_token(TokenKind::OpenBracket))),
                ']' => Some(Ok(self.make_token(TokenKind::CloseBracket))),
                '{' => Some(Ok(self.make_token(TokenKind::OpenCurly))),
                '}' => Some(Ok(self.make_token(TokenKind::CloseCurly))),
                '\\' => Some(Ok(self.make_token(TokenKind::Backslash))),
                '#' => Some(Ok(self.make_token(TokenKind::Hashtag))),
                '@' => Some(Ok(self.make_token(TokenKind::At))),
                // Operators
                '.' => Some(Ok(self.make_token(TokenKind::OpDot))),
                '=' => {
                    if self.peek() == '=' {
                        Some(Ok(self.make_token(TokenKind::OpEq)))
                    } else {
                        Some(Ok(self.make_token(TokenKind::Eq)))
                    }
                }
                '!' => {
                    if self.peek() == '=' {
                        Some(Ok(self.make_token(TokenKind::OpNeq)))
                    } else {
                        Some(Err(ParseError::UnexpectedCharacter(cur)))
                    }
                }
                '>' => {
                    if self.peek() == '=' {
                        Some(Ok(self.make_token(TokenKind::OpGtEq)))
                    } else {
                        Some(Ok(self.make_token(TokenKind::OpGt)))
                    }
                }
                '<' => {
                    if self.peek() == '=' {
                        Some(Ok(self.make_token(TokenKind::OpLtEq)))
                    } else {
                        Some(Ok(self.make_token(TokenKind::OpLt)))
                    }
                }
                '+' => {
                    if self.peek() == '+' {
                        Some(Ok(self.make_token(TokenKind::OpInc)))
                    } else {
                        Some(Ok(self.make_token(TokenKind::OpAdd)))
                    }
                }
                '-' => {
                    if self.peek() == '-' {
                        Some(Ok(self.make_token(TokenKind::OpDec)))
                    } else {
                        Some(Ok(self.make_token(TokenKind::OpSub)))
                    }
                }
                '*' => Some(Ok(self.make_token(TokenKind::OpMul))),
                '/' => Some(Ok(self.make_token(TokenKind::OpDiv))),
                '%' => Some(Ok(self.make_token(TokenKind::OpMod))),
                // Literals
                '"' => Some(self.lex_string()),
                cur if is_valid_id_start(cur) => Some(self.lex_id_or_keyword()),
                '0'..='9' => Some(self.lex_number()),
                _ => Some(Err(ParseError::UnexpectedCharacter(cur))),
            }
        }
    }
}

pub fn make_lexer<'a>(code: &'a String) -> Lexer<'a> {
    Lexer {
        code: code.chars().peekable(),
        length: code.len(),
        start: 0,
        pos: 0,
        line: 1,
        cur: ' ',
        buffer: Default::default(),
    }
}

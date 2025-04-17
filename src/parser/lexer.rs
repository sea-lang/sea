use std::{iter::Peekable, str::Chars};

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
}

impl<'a> Lexer<'a> {
    fn skip(&mut self) {
        self.cur = self.peek();
        self.code.next();
        self.pos += 1;
    }

    fn advance(&mut self) -> char {
        let it = self.peek();
        self.skip();
        it
    }

    fn peek(&mut self) -> char {
        self.code.peek().map_or('\0', |it| *it)
    }

    fn is_done(&self) -> bool {
        self.pos >= self.length
    }

    fn skip_whitespace(&mut self) {
        loop {
            let ch = self.peek();
            match ch {
                '\n' => {
                    self.skip();
                    self.line += 1;
                }
                ch if ch.is_whitespace() => self.skip(),
                _ => return,
            }
        }
    }

    fn lex_string(&mut self) -> Result<Token, ParseError> {
        let mut s: String = Default::default();

        self.start += 1;

        while self.peek() != '"' && !self.is_done() {
            if self.cur == '\n' {
                self.line += 1;
            }
            self.skip();
            s.push(self.cur);
        }

        if self.is_done() {
            Err(ParseError::UnterminatedString)
        } else {
            self.skip(); // skip the closing quote
            Ok(Token {
                kind: TokenKind::String,
                start: self.start,
                len: self.pos - self.start,
                text: s,
            })
        }
    }

    pub fn next_token(&mut self) -> Option<Result<Token, ParseError>> {
        self.skip_whitespace();

        self.start = self.pos;

        let cur = self.advance();

        if self.is_done() {
            None
        } else {
            match cur {
                '"' => Some(self.lex_string()),
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
    }
}

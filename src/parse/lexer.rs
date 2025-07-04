use std::{collections::HashMap, iter::Peekable, path::PathBuf, str::Chars, sync::LazyLock};

use crate::{parse::error::LexError, util};

use super::{
    error::LexErrorKind,
    token::{Token, TokenKind},
};

pub struct Lexer<'a> {
    pub file: PathBuf,             // path to the file being lexed
    pub source: &'a str,           // source code
    pub code: Peekable<Chars<'a>>, // source code being lexed
    pub start: usize,              // index at which the current token started
    pub column: usize,             // column of the current token
    pub pos: usize,                // index to the current character
    pub line: usize,               // current line that the lexer is on
    pub prev_token: Token,         // the previous token emitted
    cur: char,                     // current character
    prev: char,                    // previous character
    buffer: String,                // all characters since `start`
}

static KEYWORDS: LazyLock<HashMap<&str, TokenKind>> = LazyLock::new(|| {
    HashMap::from([
        ("use", TokenKind::KwUse),
        ("pkg", TokenKind::KwPkg),
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
        ("continue", TokenKind::KwContinue),
        ("break", TokenKind::KwBreak),
        ("defer", TokenKind::KwDefer),
        ("new", TokenKind::KwNew),
        ("ref", TokenKind::KwRef),
        ("as", TokenKind::KwAs),
        ("to", TokenKind::KwTo),
        ("in", TokenKind::KwIn),
        ("def", TokenKind::KwDef),
        ("tag", TokenKind::KwTag),
        ("pragma", TokenKind::KwPragma),
        ("switch", TokenKind::KwSwitch),
        ("case", TokenKind::KwCase),
        ("fall", TokenKind::KwFall),
        ("not", TokenKind::OpNot),
        ("and", TokenKind::OpAnd),
        ("or", TokenKind::OpOr),
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
    pub fn new(file: PathBuf, code: &'a String) -> Self {
        Lexer {
            file,
            source: code,
            code: code.chars().peekable(),
            start: 0,
            column: 1,
            pos: 0,
            line: 1,
            prev_token: Default::default(),
            cur: ' ',
            prev: ' ',
            buffer: Default::default(),
        }
    }

    // Gets the provided line, along with the one before and the one after it. Used for error messages.
    pub fn get_lines(&self, line: usize) -> Vec<(usize, String)> {
        util::get_lines_from_str(self.source, line)
    }

    fn skip_no_buffer(&mut self) {
        self.prev = self.cur;
        self.cur = self.peek();
        self.code.next();
        self.pos += 1;
        self.column += 1;
    }

    fn skip(&mut self) {
        self.skip_no_buffer();
        self.buffer.push(self.cur);
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
        self.cur == '\0'
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            start: self.start,
            column: self.column, // - (self.pos - self.start),
            len: self.pos - self.start,
            text: self.buffer.clone(),
            line: self.line,
        }
    }

    fn make_error_token(&self) -> Token {
        Token {
            kind: TokenKind::Error,
            start: self.start,
            column: self.column,
            len: 1,
            text: "".to_string(),
            line: self.line,
        }
    }

    fn make_error(&self, error: LexErrorKind) -> LexError {
        LexError {
            error,
            token: self.make_error_token(),
        }
    }

    fn make_error_with_token(&self, error: LexErrorKind, token: Token) -> LexError {
        LexError { error, token }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let ch = self.peek();
            match ch {
                '\n' => {
                    self.skip_no_buffer();
                    self.line += 1;
                    self.column = 1;
                }
                ch if ch.is_whitespace() => self.skip(),
                _ => return,
            }
        }
    }

    fn lex_string(&mut self) -> Result<Token, LexError> {
        self.start += 1; // skip the opening quote
        let start_line = self.line;
        let start_column = self.column;
        self.buffer.remove(0); // remove the opening quote from the buffer

        while self.peek() != '"' && !self.is_done() {
            if self.peek() == '\\' {
                self.skip();
            }
            if self.cur == '\n' {
                self.line += 1;
                self.column = 1;
            }
            self.skip();
        }

        if self.is_done() {
            Err(self.make_error_with_token(
                LexErrorKind::UnterminatedString,
                Token {
                    kind: TokenKind::String,
                    start: self.start,
                    column: start_column - 1,
                    len: 1,
                    text: "".to_string(),
                    line: start_line,
                },
            ))
        } else {
            self.skip_no_buffer(); // eat the closing quote
            Ok(self.make_token(TokenKind::String))
        }
    }

    fn lex_c_string(&mut self) -> Result<Token, LexError> {
        self.start += 1;
        self.advance(); // skip the `c`
        self.buffer.remove(0); // remove the `c` from the buffer

        match self.lex_string() {
            Ok(it) => Ok(Token {
                kind: TokenKind::CString,
                ..it
            }),
            Err(it) => Err(it),
        }
    }

    fn lex_char(&mut self) -> Result<Token, LexError> {
        self.start += 1;

        let start_line = self.line;
        let start_column = self.column;

        self.advance(); // skip the ```
        self.buffer.remove(0); // remove the ``` from the buffer

        while self.peek() != '`' && !self.is_done() {
            self.skip();
        }

        if self.is_done() {
            Err(self.make_error_with_token(
                LexErrorKind::UnterminatedChar,
                Token {
                    kind: TokenKind::String,
                    start: self.start,
                    column: start_column - 1,
                    len: 1,
                    text: "".to_string(),
                    line: start_line,
                },
            ))
        } else {
            self.skip_no_buffer(); // eat the closing ```
            Ok(self.make_token(TokenKind::Character))
        }
    }

    fn lex_number(&mut self) -> Result<Token, LexError> {
        let mut is_float = false;

        while self.peek().is_alphanumeric() || self.peek() == '_' || self.peek() == '.' {
            if self.peek() == '.' {
                // prevent two dots in one float
                if is_float {
                    return Err(self.make_error(LexErrorKind::UnexpectedCharacter('.')));
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

    fn lex_id_or_keyword(&mut self) -> Result<Token, LexError> {
        while is_valid_id(self.peek()) {
            self.skip();
        }

        // Namespaces
        if self.peek() == '\'' {
            self.advance();
            if self.peek() == '\'' {
                return Err(self.make_error(LexErrorKind::UnexpectedCharacter('\'')));
            }
            while is_valid_id(self.peek()) {
                self.skip();
                if self.peek() == '\'' {
                    self.advance();
                    if self.peek() == '\'' {
                        return Err(self.make_error(LexErrorKind::UnexpectedCharacter('\'')));
                    }
                }
            }
        }

        if KEYWORDS.contains_key(self.buffer.as_str()) {
            Ok(self.make_token(KEYWORDS[self.buffer.as_str()]))
        } else {
            Ok(self.make_token(TokenKind::Identifier))
        }
    }

    fn lex_raw_block(&mut self) -> Result<Token, LexErrorKind> {
        let mut depth = 1;

        // Brackets may be unbalanced if they're used in strings, character
        // literals, or comments in the raw C code, so we have to track those
        // I'm willing to bet there's a better way to do this
        let mut in_string = false;
        let mut in_char = false;
        let mut in_single_line_comment = false;
        let mut in_multiline_comment = false;

        // The only character in the buffer right now should be an opening brace
        // (`[`) which we don't want, so we can clear that now
        self.buffer.clear();

        loop {
            match self.peek() {
                '"' if !in_single_line_comment
                    && !in_multiline_comment
                    && !in_char
                    && self.cur != '\\' =>
                {
                    in_string = !in_string
                }
                '\'' if !in_single_line_comment
                    && !in_multiline_comment
                    && !in_string
                    && self.cur != '\\' =>
                {
                    in_char = !in_char
                }
                '/' if !in_string && !in_char => {
                    if self.cur == '/' && !in_multiline_comment {
                        in_single_line_comment = true;
                    } else if self.cur == '*' && !in_single_line_comment {
                        in_multiline_comment = false;
                    }
                }
                '*' if self.cur == '/' && !in_string && !in_char && !in_single_line_comment => {
                    in_multiline_comment = true;
                }
                '[' if !in_single_line_comment
                    && !in_multiline_comment
                    && !in_string
                    && !in_char =>
                {
                    depth += 1
                }
                ']' if !in_single_line_comment
                    && !in_multiline_comment
                    && !in_string
                    && !in_char =>
                {
                    depth -= 1;
                    if depth == 0 {
                        self.skip_no_buffer(); // skip the closing `]`
                        break;
                    }
                }
                '\n' => {
                    self.line += 1;
                    if in_single_line_comment {
                        in_single_line_comment = false;
                    }
                }
                '\0' => return Err(LexErrorKind::UnterminatedRawBlock),
                _ => {}
            }
            self.skip();
        }

        Ok(self.make_token(TokenKind::LiteralText))
    }

    fn get_next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        self.buffer.clear();
        self.start = self.pos;

        let cur = self.advance();

        if self.prev_token.kind == TokenKind::KwRaw {
            if cur != '[' {
                Err(self.make_error(LexErrorKind::ExpectedCharacter('[', cur)))
            } else {
                Ok(self.lex_raw_block().unwrap())
            }
        } else {
            match cur {
                // Common/misc symbols
                ',' => Ok(self.make_token(TokenKind::Comma)),
                ':' => Ok(self.make_token(TokenKind::Colon)),
                ';' => Ok(self.make_token(TokenKind::Semicolon)),
                '^' => Ok(self.make_token(TokenKind::Pointer)),
                '(' => Ok(self.make_token(TokenKind::OpenParen)),
                ')' => Ok(self.make_token(TokenKind::CloseParen)),
                '[' => Ok(self.make_token(TokenKind::OpenBracket)),
                ']' => Ok(self.make_token(TokenKind::CloseBracket)),
                '{' => Ok(self.make_token(TokenKind::OpenCurly)),
                '}' => Ok(self.make_token(TokenKind::CloseCurly)),
                '\\' => Ok(self.make_token(TokenKind::Backslash)),
                '#' => Ok(self.make_token(TokenKind::Hashtag)),
                '@' => Ok(self.make_token(TokenKind::At)),
                // Operators
                '.' => Ok(self.make_token(TokenKind::OpDot)),
                '=' => match self.peek() {
                    '=' => {
                        self.skip();
                        Ok(self.make_token(TokenKind::OpEq))
                    }
                    _ => Ok(self.make_token(TokenKind::Eq)),
                },
                '!' => match self.peek() {
                    '=' => {
                        self.skip();
                        Ok(self.make_token(TokenKind::OpNeq))
                    }
                    _ => Err(self.make_error(LexErrorKind::UnexpectedCharacter(cur))),
                },
                '>' => match self.peek() {
                    '=' => {
                        self.skip();
                        Ok(self.make_token(TokenKind::OpGtEq))
                    }
                    _ => Ok(self.make_token(TokenKind::OpGt)),
                },
                '<' => match self.peek() {
                    '=' => {
                        self.skip();
                        Ok(self.make_token(TokenKind::OpLtEq))
                    }
                    _ => Ok(self.make_token(TokenKind::OpLt)),
                },
                '+' => match self.peek() {
                    '+' => {
                        self.skip();
                        Ok(self.make_token(TokenKind::OpInc))
                    }
                    _ => Ok(self.make_token(TokenKind::OpAdd)),
                },
                '-' => match self.peek() {
                    '-' => {
                        self.skip();
                        Ok(self.make_token(TokenKind::OpDec))
                    }
                    '>' => {
                        self.skip();
                        Ok(self.make_token(TokenKind::Arrow))
                    }
                    _ => Ok(self.make_token(TokenKind::OpSub)),
                },
                '*' => Ok(self.make_token(TokenKind::OpMul)),
                '/' => {
                    if self.peek() == '/' {
                        while self.peek() != '\n' {
                            self.skip_no_buffer()
                        }
                        return self.get_next_token();
                    } else if self.peek() == '*' {
                        let mut depth = 1;
                        while depth > 0 {
                            self.skip_no_buffer();
                            if self.cur == '\n' {
                                self.line += 1;
                                self.column = 1;
                            } else if self.cur == '*' && self.peek() == '/' {
                                depth -= 1;
                            } else if self.cur == '/' && self.peek() == '*' {
                                depth += 1;
                            }
                        }
                        self.skip_no_buffer(); // Skip the ending `/`
                        return self.get_next_token();
                    } else {
                        Ok(self.make_token(TokenKind::OpDiv))
                    }
                }
                '%' => Ok(self.make_token(TokenKind::OpMod)),
                cur if cur == '|' && self.peek() == '>' => Ok(self.make_token(TokenKind::OpPipe)),
                // Literals
                '"' => self.lex_string(),
                'c' if self.peek() == '"' => self.lex_c_string(),
                '`' => self.lex_char(),
                cur if is_valid_id_start(cur) => self.lex_id_or_keyword(),
                '0'..='9' => self.lex_number(),
                '\0' => Ok(self.make_token(TokenKind::Eof)),
                _ => Err(self.make_error(LexErrorKind::UnexpectedCharacter(cur))),
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        let tok = self.get_next_token();
        if tok.as_ref().is_ok() {
            self.prev_token = tok.as_ref().unwrap().clone();
        }
        tok
    }
}

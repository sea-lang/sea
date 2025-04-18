use std::{collections::HashMap, sync::LazyLock};

use super::token::TokenKind;

#[derive(Debug, Clone, Copy)]
pub enum OperatorKind {
    Ref,
    Deref,
    Dot,
    As,
    Assign,
    And,
    Or,
    Not,
    Eq,
    Neq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Inc,
    Dec,
}

#[derive(Clone, Copy)]
pub struct Operator {
    pub kind: OperatorKind,
    pub prec: u8,
    pub assoc: Associativity,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    Left,
    Right,
}

// Pulled from https://en.cppreference.com/w/c/language/operator_precedence
pub const OPERATORS: LazyLock<HashMap<TokenKind, Operator>> = LazyLock::new(|| {
    HashMap::from([
        // . as
        TokenKind::OpDot.op(0, Associativity::Left),
        TokenKind::KwAs.op(0, Associativity::Right),
        // + -
        TokenKind::OpAdd.op(1, Associativity::Left),
        TokenKind::OpSub.op(1, Associativity::Left),
        // * / %
        TokenKind::OpMul.op(2, Associativity::Left),
        TokenKind::OpDiv.op(2, Associativity::Left),
        TokenKind::OpMod.op(2, Associativity::Left),
        // < <= > >=
        TokenKind::OpGt.op(6, Associativity::Left),
        TokenKind::OpGtEq.op(6, Associativity::Left),
        TokenKind::OpLt.op(6, Associativity::Left),
        TokenKind::OpLtEq.op(6, Associativity::Left),
        // == !=
        TokenKind::OpEq.op(7, Associativity::Left),
        TokenKind::OpNeq.op(7, Associativity::Left),
        // and or
        TokenKind::OpAnd.op(11, Associativity::Left),
        TokenKind::OpOr.op(12, Associativity::Left),
    ])
});

impl TokenKind {
    pub fn is_operator(&self) -> bool {
        OPERATORS.contains_key(self)
    }

    pub fn get_operator(&self) -> Option<Operator> {
        if OPERATORS.contains_key(self) {
            Some(OPERATORS[self])
        } else {
            None
        }
    }

    pub fn get_operator_kind(&self) -> Option<OperatorKind> {
        Some(match self {
            TokenKind::OpDot => OperatorKind::Dot,
            TokenKind::KwAs => OperatorKind::As,
            TokenKind::Eq => OperatorKind::Assign,
            TokenKind::OpAdd => OperatorKind::Add,
            TokenKind::OpSub => OperatorKind::Sub,
            TokenKind::OpMul => OperatorKind::Mul,
            TokenKind::OpDiv => OperatorKind::Div,
            TokenKind::OpMod => OperatorKind::Mod,
            TokenKind::OpGt => OperatorKind::Gt,
            TokenKind::OpGtEq => OperatorKind::GtEq,
            TokenKind::OpLt => OperatorKind::Lt,
            TokenKind::OpLtEq => OperatorKind::LtEq,
            TokenKind::OpEq => OperatorKind::Eq,
            TokenKind::OpNeq => OperatorKind::Neq,
            TokenKind::OpAnd => OperatorKind::And,
            TokenKind::OpOr => OperatorKind::Or,
            _ => return None,
        })
    }

    // shorthand used to make `OPERATORS`
    fn op(&self, prec: u8, assoc: Associativity) -> (TokenKind, Operator) {
        (
            *self,
            Operator {
                kind: self.get_operator_kind().unwrap(),
                prec,
                assoc,
            },
        )
    }
}

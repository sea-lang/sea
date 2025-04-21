use std::{collections::HashMap, sync::LazyLock};

use super::token::TokenKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Negate,
}

#[derive(Clone, Copy)]
pub struct Operator {
    pub kind: OperatorKind,
    pub prec: u8,
    pub pos: Position,
    pub assoc: Associativity,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Prefix,
    Infix,
    Postfix,
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
        TokenKind::OpDot.li(1), // expr.expr
        TokenKind::KwAs.ri(1),  // expr as type
        // + -
        TokenKind::OpAdd.ri(2), // expr + expr
        TokenKind::OpSub.ri(2), // expr - expr
        // * / %
        TokenKind::OpMul.li(3), // expr * expr
        TokenKind::OpDiv.li(3), // expr / expr
        TokenKind::OpMod.li(3), // expr % expr
        // > >= < <=
        TokenKind::OpGt.li(6),   // expr > expr
        TokenKind::OpGtEq.li(6), // expr >= expr
        TokenKind::OpLt.li(6),   // expr < expr
        TokenKind::OpLtEq.li(6), // expr <= expr
        // == !=
        TokenKind::OpEq.li(7),  // expr == expr
        TokenKind::OpNeq.li(7), // expr != expr
        // and or
        TokenKind::OpAnd.li(11), // expr and expr
        TokenKind::OpOr.li(12),  // expr or expr
        // =
        TokenKind::Eq.ri(14), // expr = expr
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
    fn op(&self, prec: u8, pos: Position, assoc: Associativity) -> (TokenKind, Operator) {
        (
            *self,
            Operator {
                kind: self.get_operator_kind().expect(
                    format!("internal error: get_operator_kind({:?}) == None", self).as_str(),
                ),
                prec,
                pos,
                assoc,
            },
        )
    }

    // shorthand for left associative infix operators
    fn li(&self, prec: u8) -> (TokenKind, Operator) {
        self.op(prec, Position::Infix, Associativity::Left)
    }

    // shorthand for right associative infix operators
    fn ri(&self, prec: u8) -> (TokenKind, Operator) {
        self.op(prec, Position::Infix, Associativity::Right)
    }
}

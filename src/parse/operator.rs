use std::{collections::HashMap, sync::LazyLock};

use super::token::TokenKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorKind {
    Ref,
    Deref,
    Dot,
    Pkg,
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
    Index,
    Invoke,
}

#[derive(Clone, Copy, Debug)]
pub struct Operator {
    pub kind: OperatorKind,
    pub prec: Precedence,
    pub pos: Position,
    pub assoc: Associativity,
}

pub type Precedence = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    Prefix,
    Infix,
    Postfix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Associativity {
    Left,
    Right,
}

pub const OPERATORS: LazyLock<HashMap<TokenKind, Operator>> = LazyLock::new(|| {
    HashMap::from([
        // . as
        TokenKind::OpDot.li(1000), // expr.expr
        TokenKind::KwAs.ri(1000),  // expr as type
        // and or
        TokenKind::OpAnd.li(500), // expr and expr
        TokenKind::OpOr.li(500),  // expr or expr
        // > >= < <=
        TokenKind::OpGt.li(400),   // expr > expr
        TokenKind::OpGtEq.li(400), // expr >= expr
        TokenKind::OpLt.li(400),   // expr < expr
        TokenKind::OpLtEq.li(400), // expr <= expr
        // == !=
        TokenKind::OpEq.li(300),  // expr == expr
        TokenKind::OpNeq.li(300), // expr != expr
        // * / %
        TokenKind::OpMul.li(200), // expr * expr
        TokenKind::OpDiv.li(200), // expr / expr
        TokenKind::OpMod.li(200), // expr % expr
        // + -
        TokenKind::OpAdd.ri(100), // expr + expr
        TokenKind::OpSub.ri(100), // expr - expr
        // =
        TokenKind::Eq.ri(0), // expr = expr
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
            TokenKind::OpNot => OperatorKind::Not,
            // TokenKind::OpSub => OperatorKind::Negate,
            TokenKind::KwRef => OperatorKind::Ref,
            TokenKind::Pointer => OperatorKind::Deref,
            TokenKind::OpenParen => OperatorKind::Invoke,
            _ => return None,
        })
    }

    // shorthand used to make `OPERATORS`
    fn op(&self, prec: Precedence, pos: Position, assoc: Associativity) -> (TokenKind, Operator) {
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
    fn li(&self, prec: Precedence) -> (TokenKind, Operator) {
        self.op(prec, Position::Infix, Associativity::Left)
    }

    // shorthand for right associative infix operators
    fn ri(&self, prec: Precedence) -> (TokenKind, Operator) {
        self.op(prec, Position::Infix, Associativity::Right)
    }
}

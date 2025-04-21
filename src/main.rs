use std::{fs, path::PathBuf, str::FromStr};

use parse::{lexer::make_lexer, parser::Parser, polish_notation::PolishNodeTree};
use sandbox::Sandbox;

pub mod error;
pub mod hashtags;
pub mod parse;
pub mod sandbox;

fn main() {
    // let mut sandbox = Sandbox::new();
    // sandbox.start();

    let path = PathBuf::from_str("test.sea").unwrap();
    let code = fs::read_to_string(&path).unwrap();

    // let mut lexer = make_lexer(PathBuf::new(), &code);
    // while let Some(tok) = lexer.next_token() {
    //     println!("tok: {:?}", tok);
    // }

    let mut parser = Parser::make_parser(make_lexer(path, &code));
    parser.parse().pretty_print();

    // let expr = "thing(1 + 2, -3 + 4)^.2".to_string();
    // println!("Expression: {expr}");
    // let mut expr_parser = Parser::make_parser(make_lexer(PathBuf::new(), &expr));
    // expr_parser.advance();
    // println!(
    //     "Polish Notation: {}",
    //     PolishNodeTree::from_node(expr_parser.parse_expression()).unwrap()
    // );
}

use std::{fs, path::PathBuf, str::FromStr};

use parse::{lexer::make_lexer, parser::Parser};

pub mod error;
pub mod hashtags;
pub mod parse;

fn main() {
    let path = PathBuf::from_str("test.sea").unwrap();
    let code = fs::read_to_string(&path).unwrap();

    // let mut lexer = make_lexer(PathBuf::new(), &code);
    // while let Some(tok) = lexer.next_token() {
    //     println!("tok: {:?}", tok);
    // }

    let mut parser = Parser::make_parser(make_lexer(path, &code));
    parser.parse().pretty_print();

    // let expr = "(1 % 2 == 0) as u8 * 2 + 3".to_string();
    // println!("Expression: {expr}");
    // let mut expr_parser = Parser::make_parser(make_lexer(PathBuf::new(), &expr));
    // expr_parser.advance();
    // println!(
    //     "Polish Notation: {}",
    //     PolishNodeTree::from_node(expr_parser.parse_expression()).unwrap()
    // );
}

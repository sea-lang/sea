use parse::{lexer::make_lexer, parser::make_parser};

pub mod error;
pub mod hashtags;
pub mod parse;

fn main() {
    let code: String = "
        use std/io

        fun something(
            a_really_weirdly_typed_parameter: ^^^fun(int, ^char[]): int,
            not_a_weird_parameter: int
        ): int[3][3] { }

        fun main(argc: int, argv: ^char[]): int {
            ret 0
        }
    "
    .to_string();

    // let mut lexer = make_lexer(&code);
    // while let Some(tok) = lexer.next_token() {
    //     println!("tok: {:?}", tok);
    // }

    let mut parser = make_parser(make_lexer(&code));

    match parser.parse() {
        Ok(node) => node.pretty_print(0),
        Err(why) => eprintln!("error: {}", why),
    }
}

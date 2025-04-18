use parse::{lexer::make_lexer, parser::make_parser, polish_notation::PolishNodeTree};

pub mod error;
pub mod hashtags;
pub mod parse;

fn main() {
    let code: String = "
        use std/io
        use a_thing

        fun $omething(
            a_really_weirdly_typed_parameter: ^^^fun(int, ^char[]): int,
            not_a_weird_parameter: int
        ): int[3][3] { }

        #inline
        fun fake() -> ret new Fake()

        #(inline, static)
        fun another_fake() -> ret new AnotherFake(0, true, false, {
            ret 1_000_000
        })

        fun main(argc: int, argv: ^char[]): int {
            ret 1 + 2 + 3 * 4 as u8
            ret 0x02
            ret 0b11
            ret 4.0
        }
    "
    .to_string();

    // let mut lexer = make_lexer(&code);
    // while let Some(tok) = lexer.next_token() {
    //     println!("tok: {:?}", tok);
    // }

    let mut parser = make_parser(make_lexer(&code));

    match parser.parse() {
        Ok(node) => node.pretty_print(),
        Err(why) => eprintln!("error: {}", why),
    }

    let expr = "(1 % 2 == 0) as u8 * 2 + 3".to_string();
    println!("Expression: {expr}");
    let mut expr_parser = make_parser(make_lexer(&expr));
    expr_parser.advance();
    match expr_parser.parse_expression() {
        Ok(node) => println!(
            "Polish Notation: {}",
            PolishNodeTree::from_node(node).unwrap()
        ),
        Err(why) => eprintln!("error: {}", why),
    }
}

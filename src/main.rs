use parser::lexer::make_lexer;

pub mod ast;
pub mod error;
pub mod hashtags;
pub mod parser;
pub mod type_;

fn main() {
    let source = "use std/io

fun main(): int {
	println(\"Hello, World!\")
	ret 0
}
"
    .to_string();

    // let source = "\"A string!\" \"another string!!!\" \"a
    // multiline string\""
    //     .to_string();

    let mut lexer = make_lexer(&source);

    while let Some(it) = lexer.next_token() {
        match it {
            Ok(token) => println!("token: {}", token),
            Err(why) => println!("parsing error: {}", why.to_string()),
        }
    }
}

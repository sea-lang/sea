use parser::lexer::make_lexer;

pub mod error;
pub mod parser;

fn main() {
    let source = "\"A string!\" \"another string!!!\" \"a
	multiline string\""
        .to_string();

    let mut lexer = make_lexer(&source);

    while let Some(it) = lexer.next_token() {
        match it {
            Ok(token) => println!("token: {}", token),
            Err(why) => println!("parsing error: {}", why.to_string()),
        }
    }
}

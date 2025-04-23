use std::{fs, path::PathBuf, process::exit};

use backend::{backend::Backend, backends::c::CBackend};
use compile::compiler::Compiler;
use parse::{lexer::make_lexer, parser::Parser};

pub mod backend;
pub mod compile;
pub mod error;
pub mod hashtags;
pub mod parse;
pub mod sandbox;

mod flags {
    use std::path::PathBuf;

    xflags::xflags!(
        cmd app {
            cmd compile c {
                /// The file to compile
                required input: PathBuf
                /// The path to the output file
                optional -o, --output output: PathBuf
                /// Executes the program after compilation
                optional -r, --run
                /// Arguments to pass to the program, only applies when --run is passed
                repeated -a, --arg arg: String
                /// Toggles optimizations for production builds
                optional -p, --prod
                /// The C compiler to build with
                optional -c, --cc cc: String
                /// Arguments for the C compiler
                optional -f, --ccflags ccflags: String
                /// Paths to search for libraries
                optional -l, --libpaths libpaths: String
                /// Path to the standard library
                optional -s, --std std: String
                /// Disables implicit `use std`
                optional -S, --nostd
                /// Prints the AST
                optional --print-ast
                /// Skip C compilation
                optional -n, --nobuild
            }
            cmd sandbox s {
            }
        }
    );
}

fn throw(msg: &str) -> ! {
    println!("\x1b[1;31merror:\x1b[0m {msg}");
    exit(1);
}

fn compile(flags: flags::Compile) {
    let path = || flags.input.clone(); // TODO: this is hacky and there's absolutely a better way to do it
    let c_output_path = PathBuf::from(".sea/build/output.c");
    let executable_path = flags
        .output
        .unwrap_or_else(|| PathBuf::from(".sea/build/main"));

    println!("\x1b[35m: Compiling Sea\x1b[0m");

    // Load the source code
    let code = fs::read_to_string(path()).unwrap();

    // Parse
    let mut parser = Parser::make_parser(make_lexer(path(), &code));
    let program = parser.parse();

    if flags.print_ast {
        // Print AST
        program.pretty_print();
    }

    // Make compiler and backend
    let compiler = Compiler::new(path(), c_output_path.clone(), parser);
    let mut backend = CBackend::new(compiler);

    // Write output C code
    backend.write(program);

    // Compile C code
    if !flags.nobuild {
        let compile_res = compile::run_compile_cmds(c_output_path, executable_path.clone());
        if compile_res.is_err() {
            throw(compile_res.err().unwrap().as_str());
        }
    }

    // Execute code
    if flags.run {
        let run_res = compile::run_executable(executable_path, flags.arg);
        if run_res.is_err() {
            throw(run_res.err().unwrap().as_str());
        }
    }
}

fn sandbox(_flags: flags::Sandbox) {
    let mut sandbox = sandbox::Sandbox::new();
    sandbox.start();
}

fn main() {
    let flags = flags::App::from_env_or_exit();

    match flags.subcommand {
        flags::AppCmd::Compile(args) => compile(args),
        flags::AppCmd::Sandbox(args) => sandbox(args),
    }

    // let mut lexer = make_lexer(PathBuf::new(), &code);
    // while let Some(tok) = lexer.next_token() {
    //     println!("tok: {:?}", tok);
    // }

    // let expr = "thing(1 + 2, -3 + 4)^.2".to_string();
    // println!("Expression: {expr}");
    // let mut expr_parser = Parser::make_parser(make_lexer(PathBuf::new(), &expr));
    // expr_parser.advance();
    // println!(
    //     "Polish Notation: {}",
    //     PolishNodeTree::from_node(expr_parser.parse_expression()).unwrap()
    // );
}

use std::{fs, path::PathBuf, process::exit};

use backend::{backend::Backend, backends::c::CBackend};
use compile::compiler::Compiler;
use parse::{lexer::Lexer, parser::Parser};

pub mod backend;
pub mod compile;
pub mod error;
pub mod hashtags;
pub mod parse;
pub mod reef;
pub mod sandbox;
pub mod util;

mod flags {
    use std::path::PathBuf;

    xflags::xflags! {
        cmd sea {
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
                repeated -f, --ccflags ccflags: String
                /// Paths to search for libraries
                repeated -l, --libpaths libpaths: String
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
                /// Path to the standard library
                optional -s, --std std: String
            }
        }
    }
}

fn throw(msg: &str) -> ! {
    println!("\x1b[1;31merror:\x1b[0m {msg}");
    exit(1);
}

fn get_cc(is_prod: bool) -> String {
    if is_prod {
        "gcc".to_string()
    } else {
        "tcc".to_string()
    }
}

fn compile(flags: flags::Compile) {
    let path = || flags.input.clone(); // TODO: this is hacky and there's absolutely a better way to do it
    let c_output_path = PathBuf::from(".sea/build/output.c");
    let executable_path = flags
        .output
        .unwrap_or_else(|| PathBuf::from(".sea/build/main"));

    let mut libpaths: Vec<PathBuf> = vec![];
    let stdpath = PathBuf::from(flags.std.unwrap_or_else(|| "~/.sea/std/".to_string()));
    libpaths.push(stdpath);
    for it in flags.libpaths {
        libpaths.push(PathBuf::from(it))
    }

    println!("\x1b[35m: Compiling Sea\x1b[0m");

    // Load the source code
    let code = fs::read_to_string(path()).unwrap();

    // Parse
    let mut parser = Parser::new(Lexer::new(path(), &code));
    let program = parser.parse(!flags.nostd);

    if flags.print_ast {
        // Print AST
        program.pretty_print();
    }

    // Make compiler and backend
    let mut compiler = Compiler::new(path(), c_output_path.clone(), libpaths, parser);

    // This gets used in C code compilation, I make it now so that the borrow checker doesn't make me cry
    let mut cc_flags: Vec<String> = vec![];

    // Write output C code
    CBackend::new(&mut compiler).write(program);

    // Compile C code
    if !flags.nobuild {
        cc_flags.extend_from_slice(&compiler.cc_flags);
        cc_flags.extend_from_slice(&flags.ccflags);

        let compile_res = compile::run_compile_cmds(
            c_output_path,
            executable_path.clone(),
            flags.cc.unwrap_or(get_cc(flags.prod)),
            cc_flags,
        );

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

fn sandbox(flags: flags::Sandbox) {
    let mut sandbox = sandbox::Sandbox::new();
    sandbox.libpaths.push(PathBuf::from(
        flags.std.unwrap_or_else(|| "~/.sea/std/".to_string()),
    ));
    sandbox.start();
}

fn main() {
    let flags = flags::Sea::from_env_or_exit();

    match flags.subcommand {
        flags::SeaCmd::Compile(args) => compile(args),
        flags::SeaCmd::Sandbox(args) => sandbox(args),
    }
}

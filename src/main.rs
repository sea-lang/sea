use std::{fs, path::PathBuf, str::FromStr};

use clap;

use parse::{lexer::make_lexer, parser::Parser, polish_notation::PolishNodeTree};
use sandbox::Sandbox;

pub mod error;
pub mod hashtags;
pub mod parse;
pub mod sandbox;

#[derive(Debug, clap::Parser)]
#[command(name = "Sea")]
#[command(version = "0.0.1-dev")]
#[command(about = "Sea compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Compile(CompileArgs),
    Run(CompileArgs, RunArgs),
    Sandbox(SandboxArgs),
}

#[derive(clap::Args)]
struct CompileArgs {
    /// The file to compile
    file: Option<PathBuf>,
    /// The path to the output file
    #[arg(short, long, value_name = "FILE", default_value_t = "main")]
    output: PathBuf,
    /// Toggles optimizations for production builds
    #[arg(short, long)]
    prod: bool,
    /// The C compiler to build with
    #[arg(short, long, value_name = "COMPILER")]
    cc: Option<String>,
    /// Arguments for the C compiler
    #[arg(short = 'f', long, value_name = "COMPILER FLAGS")]
    ccflags: Option<String>,
    /// Paths to search for libraries
    #[arg(short, long)]
    libpaths: Option<String>,
    /// Path to the standard library
    #[arg(short, long)]
    std: String,
    /// Disables implicit `use std`
    #[arg(short = 'S', long)]
    nostd: bool,
}

#[derive(clap::Args)]
struct RunArgs {
    /// Disables rebuild
    #[arg(short, long)]
    nobuild: bool,
    // Args to pass to the program
    #[arg(short, long, value_name = "PROGRAM ARGS", default_value_t = "")]
    args: String,
}

#[derive(clap::Args)]
struct SandboxArgs {}

fn compile(args: CompileArgs) {
    let path = PathBuf::from_str(args.file).unwrap();
    let code = fs::read_to_string(&path).unwrap();

    let mut parser = Parser::make_parser(make_lexer(path, &code));
    parser.parse().pretty_print();
}

fn run(args: RunArgs) {
    unimplemented!()
}

fn sandbox(args: SandboxArgs) {
    let mut sandbox = Sandbox::new();
    sandbox.start();
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Compile(args) => compile(args),
        Commands::Run(compile_args, run_args) => {
            if !run_args.nobuild {
                compile(compile_args);
            }
            run(run_args);
        }
        Commands::Sandbox(args) => sandbox(args),
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

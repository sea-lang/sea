use std::{io::ErrorKind, path::PathBuf, process::Command};

pub mod compiler;
pub mod error;
pub mod infer;
pub mod pragmas;
pub mod symbol;
pub mod type_;

pub fn run_compile_cmds(
    c_source_path: PathBuf,
    executable_path: PathBuf,
    cc: String,
    cc_flags: Vec<String>,
) -> Result<(), String> {
    let mut compile_cmd = Command::new(&cc);

    cc_flags.iter().for_each(|it| {
        compile_cmd.arg(it);
    });
    compile_cmd.arg("-g3");
    compile_cmd.arg("-o");
    compile_cmd.arg(executable_path.to_str().unwrap());
    compile_cmd.arg(c_source_path.to_str().unwrap());

    println!(
        "\x1b[35m: Compiling C: \x1b[1;35m{} {}\x1b[0m",
        cc,
        compile_cmd
            .get_args()
            .map(|it| it.to_str().unwrap())
            .collect::<Vec<&str>>()
            .join(" ")
    );

    match compile_cmd.spawn() {
        Ok(mut child) => {
            let res = child.wait().expect("failed to wait for child");
            if !res.success() {
                let code = res.code().unwrap_or(-1);
                println!("\x1b[31m: Process exited with code: {code}\x1b[0m",);
                Err(format!("process exited with code: {code}").to_string())
            } else {
                Ok(())
            }
        }
        Err(err) => match err.kind() {
            ErrorKind::NotFound => Err(format!("command not found: `{cc}`")),
            _ => Err(format!("error during compilation: {err:?}")),
        },
    }
}

pub fn run_executable(path: PathBuf, args: Vec<String>) -> Result<(), String> {
    println!(
        "\x1b[35m: Executing: \x1b[1;35m{}\x1b[0m",
        path.to_str().unwrap()
    );

    let mut cmd = Command::new(path);
    cmd.args(args);

    match cmd.spawn() {
        Ok(mut child) => {
            let res = child.wait().expect("failed to wait for child");
            if !res.success() {
                println!(
                    "\x1b[31m: Process exited with code: {}\x1b[0m",
                    res.code().unwrap_or(-1)
                );
            }
            Ok(())
        }
        Err(err) => Err(format!("error during execution: {err:?}")),
    }
}

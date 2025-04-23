use std::{path::PathBuf, process::Command};

pub mod compiler;
pub mod error;
pub mod symbol;

pub fn run_compile_cmds(c_source_path: PathBuf, executable_path: PathBuf) -> Result<(), String> {
    println!(
        "\x1b[35m: Compiling C: \x1b[1;35mtcc -g3 -o {} {}\x1b[0m",
        executable_path.to_str().unwrap(),
        c_source_path.to_str().unwrap()
    );

    let mut compile_cmd = Command::new("tcc");
    compile_cmd.arg("-g3");
    compile_cmd.arg("-o");
    compile_cmd.arg(executable_path.to_str().unwrap());
    compile_cmd.arg(c_source_path.to_str().unwrap());

    match compile_cmd.spawn() {
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
        Err(err) => Err(format!("error during compilation: {err:?}")),
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

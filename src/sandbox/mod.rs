use std::{collections::HashMap, fs, path::PathBuf, str::FromStr, sync::LazyLock};

use text_io::read;

use crate::{
    backend::{backend::Backend, backends::c::CBackend},
    compile::{self, compiler::Compiler},
    parse::{lexer::Lexer, parser::Parser},
};

const HELP: &'static str = r#"Sandbox Commands:
  \h \help            - Show this message
  \e \echo            - Echo all lines
  \q \quit            - Quits
  \R \reset           - Reset the sandbox
  \p \pos <int|"end"> - Jump to the given line (int) or to the last line ("end")
  \u \pause           - Pause auto-compilation
  \r \read <file>     - Read from a given file
  \w \write <file>    - Write to a given file
  \P \replace         - Toggle line replacement when editing previous lines, when off, lines are shifted (default: off)
  \a \ast             - Print the AST for the sandbox's code
  \x \exec <args>     - Execute the sandbox's code using the provided arguments, if any
  \A \autoexec        - Toggle automatic execution after compilation (default: off)
  \R \args <args>     - Set args to pass to the program when no others are provided
"#;

pub struct SandboxCommand {
    pub run: Box<dyn Fn(&mut Sandbox, Vec<&str>)>,
}

fn cmd<T>(text: &'static str, run: T) -> (&'static str, SandboxCommand)
where
    T: Fn(&mut Sandbox, Vec<&str>) + 'static,
{
    (text, SandboxCommand { run: Box::new(run) })
}

fn alias(text: &'static str, alias_to: &'static str) -> (&'static str, SandboxCommand) {
    cmd(text, move |sandbox, args| {
        sandbox.eval_args(alias_to, args);
    })
}

pub const COMMANDS: LazyLock<HashMap<&'static str, SandboxCommand>> = LazyLock::new(|| {
    HashMap::from([
        cmd("help", |_sandbox, _args| println!("{}", HELP)),
        cmd("echo", |sandbox, _args| {
            for (line, text) in sandbox.lines.iter().enumerate() {
                println!("\x1b[1;35m{:>5?} |\x1b[0m {text}", line + 1);
            }
        }),
        cmd("quit", |sandbox, _args| sandbox.running = false),
        cmd("reset", |sandbox, _args| {
            sandbox.lines.clear();
            sandbox.line = 1;
            sandbox.recompile();
        }),
        cmd("pos", |sandbox, args| {
            if args.len() == 0 {
                sandbox.throw("\\pos <arg>: no argument specified");
                return;
            }

            let arg = args[0];
            if arg == "end" {
                sandbox.line = sandbox.lines.len() + 1;
            } else {
                let pos = arg.parse::<usize>();
                if pos.is_err() {
                    sandbox.throw("\\pos <arg>: arg must be an integer or `end`")
                }
                sandbox.line = pos.unwrap();
            }
        }),
        cmd("pause", |sandbox, _args| {
            sandbox.paused = !sandbox.paused;
            if sandbox.paused {
                println!("\x1b[1;35mPaused compilation.\x1b[0m");
            } else {
                println!("\x1b[1;35mUnpaused compilation.\x1b[0m");
                sandbox.recompile();
            }
        }),
        cmd("read", |sandbox, args| {
            if args.len() == 0 {
                sandbox.throw("\\read <arg>: no argument specified");
            }

            let arg = args[0];

            let mut lines = fs::read_to_string(arg)
                .unwrap()
                .lines()
                .map(|it| it.to_string())
                .collect::<Vec<String>>();
            sandbox.lines.append(&mut lines);
        }),
        cmd("write", |sandbox, args| {
            if args.len() == 0 {
                sandbox.throw("\\write <arg>: no argument specified");
            }

            let arg = args[0];

            match fs::write(arg, sandbox.lines.join("\n")) {
                Ok(_) => println!("\x1b[1;35mWrote to file successfully.\x1b[0m"),
                Err(_) => sandbox.throw("failed to write to file."),
            }
        }),
        cmd("replace", |sandbox, _args| {
            sandbox.replace_lines = !sandbox.replace_lines;
            if sandbox.replace_lines {
                println!("\x1b[1;35mReplacing lines.\x1b[0m");
            } else {
                println!("\x1b[1;35mShifting lines.\x1b[0m");
            }
        }),
        cmd("ast", |sandbox, _args| {
            let code = sandbox.get_code();
            let lexer = Lexer::new(PathBuf::from_str(".sandbox").unwrap(), &code);
            let mut parser = Parser::new(lexer);
            let program = parser.parse(true);
            program.pretty_print();
        }),
        cmd("exec", |sandbox, args| {
            if args.len() > 0 {
                sandbox.exec(
                    args.iter()
                        .map(|it| it.to_string())
                        .collect::<Vec<String>>(),
                );
            } else {
                sandbox.exec(sandbox.program_args.clone());
            }
        }),
        cmd("autoexec", |sandbox, _args| {
            sandbox.autoexec = !sandbox.autoexec;
            if sandbox.autoexec {
                println!("\x1b[1;35mToggled autoexec on.\x1b[0m");
            } else {
                println!("\x1b[1;35mToggled autoexec off.\x1b[0m");
            }
        }),
        cmd("args", |sandbox, args| {
            sandbox.program_args = args
                .iter()
                .map(|it| it.to_string())
                .collect::<Vec<String>>();
        }),
        // Aliases
        alias("h", "help"),
        alias("e", "echo"),
        alias("q", "quit"),
        alias("R", "reset"),
        alias("p", "pos"),
        alias("u", "pause"),
        alias("r", "read"),
        alias("w", "write"),
        alias("P", "replace"),
        alias("a", "ast"),
        alias("x", "exec"),
        alias("A", "autoexec"),
    ])
});

pub struct Sandbox {
    pub lines: Vec<String>,
    pub line: usize,
    pub running: bool,
    pub paused: bool,
    pub autoexec: bool,
    pub program_args: Vec<String>,
    pub replace_lines: bool,
    pub output_path: PathBuf,
    pub c_output_path: PathBuf,
}

impl Sandbox {
    pub fn prompt(&self) {
        print!(
            "\x1b[1;34m[{}{}]:\x1b[0m ",
            if self.paused { "P " } else { "" },
            self.line
        );
    }

    pub fn throw(&mut self, msg: &str) {
        println!("\x1b[1;31merror:\x1b[0m {msg}");
    }

    pub fn set_line(&mut self, line: usize) -> bool {
        if line > self.line {
            false
        } else {
            self.line = line;
            true
        }
    }

    pub fn get_code(&mut self) -> String {
        self.lines.join("\n")
    }

    pub fn shlex(&mut self, text: String) -> Vec<String> {
        let mut args: Vec<String> = vec![];
        let mut buf: String = Default::default();
        let mut in_str: bool = false;
        let mut prev: char = ' ';
        let mut itr = text.chars().peekable();

        while let Some(ch) = itr.next() {
            match ch {
                '"' if prev != '\\' => in_str = !in_str,
                ' ' => {
                    loop {
                        match itr.peek() {
                            Some(it) => {
                                if *it != ' ' {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    if !buf.is_empty() {
                        args.push(buf.clone());
                        buf.clear();
                    }
                }
                _ => buf.push(ch),
            }
            prev = ch;
        }

        if !buf.is_empty() {
            args.push(buf);
        }

        args
    }

    pub fn recompile(&mut self) {
        let code = self.get_code();
        let lexer = Lexer::new(PathBuf::from_str(".sandbox").unwrap(), &code);
        let mut parser = Parser::new(lexer);
        let program = parser.parse(true);

        fs::create_dir_all(self.c_output_path.clone().parent().unwrap())
            .expect("failed to mkdirs .sea/sandbox/");

        // Make compiler and backend
        let compiler = Compiler::new(
            self.output_path.clone(),
            self.c_output_path.clone(),
            vec![], //TODO
            parser,
        );
        let mut backend = CBackend::new(compiler);

        // Write output C code
        backend.write(program);

        // Exec
        if self.autoexec {
            self.exec(self.program_args.clone())
        }
    }

    pub fn exec(&mut self, args: Vec<String>) {
        println!("\x1b[35m: Compiling Sea\x1b[0m");
        self.recompile();

        let compile_res = compile::run_compile_cmds(
            PathBuf::from(".sea/sandbox/program.c"),
            PathBuf::from(".sea/sandbox/program"),
            "tcc".to_string(),
            vec![],
        );
        if compile_res.is_err() {
            self.throw(compile_res.err().unwrap().as_str());
        }

        let run_res = compile::run_executable(PathBuf::from(".sea/sandbox/program"), args);
        if run_res.is_err() {
            self.throw(run_res.err().unwrap().as_str());
        }
    }

    pub fn eval_args(&mut self, cmd: &str, args: Vec<&str>) {
        if let Some(cmd) = COMMANDS.get(cmd) {
            (*cmd.run)(self, args);
        } else {
            self.throw("invalid command, see \x1b[1m\\help\x1b[0m");
        }
    }

    pub fn eval(&mut self, line: String) {
        if line.len() > 0 {
            let maybe_ch = line.chars().nth(0);
            if maybe_ch.is_some_and(|ch| ch == '\\') {
                if line == "\\" {
                    return;
                }

                let split = self.shlex(line);

                let cmd = split[0].strip_prefix('\\').unwrap();
                let args = split[1..]
                    .iter()
                    .map(|it| it.as_str())
                    .collect::<Vec<&str>>();

                self.eval_args(cmd, args);
            } else {
                let escaped = line.ends_with('\\');

                let line = if escaped {
                    line.strip_suffix('\\').unwrap()
                } else {
                    &line
                }
                .to_string();

                if self.line > self.lines.len() {
                    self.lines.push(line);
                } else {
                    if self.replace_lines {
                        self.lines[self.line - 1] = line;
                    } else {
                        self.lines.insert(self.line - 1, line);
                    }
                }
                self.line += 1;

                if !escaped && !self.paused {
                    self.recompile();
                }
            }
        }
    }

    pub fn start(&mut self) {
        self.running = true;
        while self.running {
            self.prompt();
            let line: String = read!("{}\n");
            self.eval(line);
        }
    }

    pub fn new() -> Self {
        Sandbox {
            lines: vec![],
            line: 1,
            paused: false,
            autoexec: false,
            program_args: vec![],
            running: false,
            replace_lines: false,
            output_path: PathBuf::from(".sea/sandbox/program"),
            c_output_path: PathBuf::from(".sea/sandbox/program.c"),
        }
    }
}

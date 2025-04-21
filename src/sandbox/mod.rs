use std::{collections::HashMap, fs, path::PathBuf, str::FromStr, sync::LazyLock};

use text_io::read;

use crate::parse::{ast::Node, lexer::make_lexer, parser::Parser};

const HELP: &'static str = r#"Sandbox Commands:
  \h \help            - Shows this message
  \e \echo            - Echos all lines
  \q \quit            - Quits
  \R \reset           - Resets the sandbox
  \p \pos <int|"end"> - Jump to the given line (int) or to the last line ("end")
  \u \pause           - Pause auto-compilation
  \r \read <file>     - Read from a given file
  \w \write <file>    - Write to a given file
  \P \replace         - Toggles line replacement, when off, lines are shifted (this applies when you are editing previous lines)
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
        sandbox.eval(format!("\\{} {}", alias_to, args.join(" ")));
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
            let arg = args.iter().nth(0);
            if arg.is_none() {
                sandbox.throw("\\pos <arg>: no argument specified");
            }

            if *arg.unwrap() == "end" {
                sandbox.line = sandbox.lines.len() + 1;
            } else {
                let pos = arg.unwrap().parse::<usize>();
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
                sandbox.ast.pretty_print();
            }
        }),
        cmd("read", |sandbox, args| {
            let arg = args.iter().nth(0);
            if arg.is_none() {
                sandbox.throw("\\read <arg>: no argument specified");
            }

            let mut lines = fs::read_to_string(arg.unwrap())
                .unwrap()
                .lines()
                .map(|it| it.to_string())
                .collect::<Vec<String>>();
            sandbox.lines.append(&mut lines);
        }),
        cmd("write", |sandbox, args| {
            let arg = args.iter().nth(0);
            if arg.is_none() {
                sandbox.throw("\\read <arg>: no argument specified");
            }

            match fs::write(arg.unwrap(), sandbox.lines.join("\n")) {
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
    ])
});

pub struct Sandbox {
    pub lines: Vec<String>,
    pub line: usize,
    pub running: bool,
    pub paused: bool,
    pub replace_lines: bool,
    ast: Node,
}

impl Sandbox {
    pub fn prompt(&self) {
        print!(
            "\x1b[1;34m[{}{}]:\x1b[0m ",
            if self.paused { "P " } else { "" },
            self.line
        );
    }

    pub fn throw(&mut self, msg: &'static str) {
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

    pub fn recompile(&mut self) {
        let code = self.get_code();
        let lexer = make_lexer(PathBuf::from_str(".sandbox").unwrap(), &code);
        let mut parser = Parser::make_parser(lexer);
        self.ast = parser.parse();
    }

    pub fn eval(&mut self, line: String) {
        if line.len() > 0 {
            let maybe_ch = line.chars().nth(0);
            if maybe_ch.is_some_and(|ch| ch == '\\') {
                if line == "\\" {
                    return;
                }

                let mut split = line.split(' ');

                let cmd = split.nth(0).unwrap().strip_prefix('\\').unwrap();
                let args = split.collect::<Vec<&str>>();

                if let Some(cmd) = COMMANDS.get(cmd) {
                    (*cmd.run)(self, args);
                } else {
                    self.throw("invalid command, see \x1b[1m\\help\x1b[0m");
                }
            } else {
                let escaped = line.ends_with('\\');

                let line = if escaped {
                    line.strip_suffix('\\').unwrap()
                } else {
                    &line
                }
                .to_string();

                if self.line >= self.lines.len() {
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
                    self.ast.pretty_print();
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
            ast: Node::Program(vec![]),
            line: 1,
            paused: false,
            running: false,
            replace_lines: false,
        }
    }
}

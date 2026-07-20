//! `sh` — a small, modular POSIX-ish shell (Rust, no external dependencies).
//!
//! Usage:
//!   sh              interactive REPL
//!   sh <script>     run a script file
//!   sh -c "cmd"     run a single command line

mod ast;
mod builtins;
mod executor;
mod lexer;
mod parser;
mod shell;

use shell::Shell;
use std::process::exit;

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("sh", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().collect();
    let mut sh = Shell::new();

    // Seed PWD/OLDPWD so `cd -` works out of the box.
    if let Ok(pwd) = std::env::current_dir() {
        let pwd = pwd.to_string_lossy().to_string();
        sh.set_var("PWD", &pwd);
        if sh.var("OLDPWD").is_none() {
            sh.set_var("OLDPWD", &pwd);
        }
    }

    if args.len() > 1 && args[1] == "-c" {
        if let Some(cmd) = args.get(2) {
            sh.run_line(cmd);
            exit(sh.exit_code);
        } else {
            eprintln!("sh: -c requires an argument");
            exit(2);
        }
    }

    if args.len() > 1 {
        let script = &args[1];
        if let Err(e) = sh.run_script(script) {
            eprintln!("sh: {script}: {e}");
            exit(127);
        }
        exit(sh.exit_code);
    }

    sh.run_repl();
    exit(sh.exit_code);
}

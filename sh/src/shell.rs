//! Shell state, environment management and the interactive REPL.

use crate::ast::Program;
use crate::executor;
use crate::lexer;
use crate::parser;
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, Write};

/// Mutable shell state shared across commands.
#[derive(Clone)]
pub struct Shell {
    /// Shell variables (override the process environment).
    pub vars: HashMap<String, String>,
    /// Names marked for export to child processes.
    pub exported: HashSet<String>,
    /// Command aliases.
    pub aliases: HashMap<String, String>,
    /// Exit status of the most recent command.
    pub last_status: i32,
    /// Set when `exit` is requested.
    pub should_exit: bool,
    /// Exit code to return from the process.
    pub exit_code: i32,
    /// Current input line number (for diagnostics).
    pub line: usize,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            vars: HashMap::new(),
            exported: HashSet::new(),
            aliases: HashMap::new(),
            last_status: 0,
            should_exit: false,
            exit_code: 0,
            line: 0,
        }
    }

    /// Look up a variable, checking shell vars then the process environment.
    pub fn var(&self, name: &str) -> Option<String> {
        self.vars
            .get(name)
            .cloned()
            .or_else(|| std::env::var(name).ok())
    }

    /// Set a variable in shell state and in the process environment.
    pub fn set_var(&mut self, name: &str, value: &str) {
        self.vars.insert(name.to_string(), value.to_string());
        unsafe {
            std::env::set_var(name, value);
        }
    }

    /// Remove a variable from shell state and the process environment.
    pub fn unset_var(&mut self, name: &str) {
        self.vars.remove(name);
        self.exported.remove(name);
        unsafe {
            std::env::remove_var(name);
        }
    }

    /// Mark a name as exported.
    pub fn mark_exported(&mut self, name: &str) {
        self.exported.insert(name.to_string());
        if let Some(v) = self.vars.get(name) {
            unsafe {
                std::env::set_var(name, v);
            }
        }
    }

    /// Combined environment: process environment overlaid with shell vars.
    pub fn combined_env(&self) -> HashMap<String, String> {
        let mut m: HashMap<String, String> = std::env::vars().collect();
        for (k, v) in &self.vars {
            m.insert(k.clone(), v.clone());
        }
        m
    }

    /// Expand aliases in a raw input line (single pass, non-recursive).
    fn apply_aliases(&self, line: &str) -> String {
        if self.aliases.is_empty() {
            return line.to_string();
        }
        let mut out = String::new();
        for (i, word) in line.split_whitespace().enumerate() {
            if i == 0 {
                if let Some(a) = self.aliases.get(word) {
                    out.push_str(a);
                } else {
                    out.push_str(word);
                }
            } else {
                out.push(' ');
                out.push_str(word);
            }
        }
        out
    }

    /// Execute a single line of input, updating `last_status`.
    pub fn run_line(&mut self, raw: &str) {
        let line = self.apply_aliases(raw);
        let toks = match lexer::tokenize(&line) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("sh: syntax error: {e}");
                self.last_status = 2;
                return;
            }
        };
        if toks.is_empty() {
            return;
        }
        let program: Program = match parser::parse(toks) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("sh: {e}");
                self.last_status = 2;
                return;
            }
        };
        if program.jobs.is_empty() {
            return;
        }
        self.last_status = executor::execute(self, &program);
    }

    /// Run the interactive REPL until EOF or `exit`.
    pub fn run_repl(&mut self) {
        let stdin = io::stdin();
        loop {
            print!("$ ");
            io::stdout().flush().ok();
            let mut input = String::new();
            if stdin.lock().read_line(&mut input).is_err() || input.is_empty() {
                println!();
                break;
            }
            let line = input.trim_end();
            if line.is_empty() {
                continue;
            }
            self.line += 1;
            self.run_line(line);
            if self.should_exit {
                break;
            }
        }
    }

    /// Execute the contents of a script file, line by line.
    pub fn run_script(&mut self, path: &str) -> std::io::Result<()> {
        let data = std::fs::read_to_string(path)?;
        for line in data.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            self.run_line(trimmed);
            if self.should_exit {
                break;
            }
        }
        Ok(())
    }
}

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    eprintln!("Usage: retab [OPTIONS] [FILE...]");
    eprintln!("Convert spaces to tabs.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -t TABSTOP   Tab stop positions (default: 8)");
    eprintln!("  -a           Convert all strings of spaces (not just leading)");
    eprintln!("  -h, --help   Show this help message");
    eprintln!("  -v, --version Show version");
}

fn convert_line(line: &str, tabstop: usize, all: bool) -> String {
    if all {
        let mut result = String::new();
        let mut col = 0;
        let mut space_start = None;

        for ch in line.chars() {
            if ch == ' ' {
                if space_start.is_none() {
                    space_start = Some(col);
                }
                col += 1;
            } else {
                if let Some(start) = space_start {
                    let len = col - start;
                    let mut pos = start;
                    while pos + tabstop <= start + len {
                        result.push('\t');
                        pos += tabstop;
                    }
                    let remaining = start + len - pos;
                    if remaining > 0 {
                        result.push_str(&" ".repeat(remaining));
                    }
                    space_start = None;
                }
                result.push(ch);
                if ch == '\n' || ch == '\r' {
                    col = 0;
                } else {
                    col += 1;
                }
            }
        }

        if let Some(start) = space_start {
            let len = col - start;
            let mut pos = start;
            while pos + tabstop <= start + len {
                result.push('\t');
                pos += tabstop;
            }
            let remaining = start + len - pos;
            if remaining > 0 {
                result.push_str(&" ".repeat(remaining));
            }
        }

        result
    } else {
        let mut result = String::new();
        let mut col = 0;
        let mut space_start = None;

        for ch in line.chars() {
            if ch == ' ' && space_start.is_none() {
                space_start = Some(col);
                col += 1;
            } else if ch == ' ' && space_start.is_some() {
                col += 1;
            } else {
                if let Some(start) = space_start {
                    let len = col - start;
                    let mut pos = start;
                    while pos + tabstop <= start + len {
                        result.push('\t');
                        pos += tabstop;
                    }
                    let remaining = start + len - pos;
                    if remaining > 0 {
                        result.push_str(&" ".repeat(remaining));
                    }
                    space_start = None;
                }
                result.push(ch);
                if ch == '\n' || ch == '\r' {
                    col = 0;
                } else {
                    col += 1;
                }
            }
        }

        if let Some(start) = space_start {
            let len = col - start;
            let mut pos = start;
            while pos + tabstop <= start + len {
                result.push('\t');
                pos += tabstop;
            }
            let remaining = start + len - pos;
            if remaining > 0 {
                result.push_str(&" ".repeat(remaining));
            }
        }

        result
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut tabstop: usize = 8;
    let mut all = false;
    let mut files = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-v" | "--version" => {
                println!("retab {VERSION}");
                return;
            }
            "-t" => {
                i += 1;
                if i < args.len() {
                    tabstop = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("retab: invalid tab stop '{}'", args[i]);
                        std::process::exit(1);
                    });
                    if tabstop == 0 {
                        eprintln!("retab: tab stop must be > 0");
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("retab: -t requires an argument");
                    std::process::exit(1);
                }
            }
            "-a" => all = true,
            "--" => {
                i += 1;
                while i < args.len() {
                    files.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            _ if args[i].starts_with('-') => {
                eprintln!("retab: unknown option '{}'", args[i]);
                std::process::exit(1);
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    if files.is_empty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let converted = convert_line(&line, tabstop, all);
            write!(out, "{converted}").ok();
        }
    } else {
        for path in &files {
            let file = File::open(path).unwrap_or_else(|e| {
                eprintln!("retab: cannot read '{path}': {e}");
                std::process::exit(1);
            });
            for line in BufReader::new(file).lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };
                let converted = convert_line(&line, tabstop, all);
                write!(out, "{converted}").ok();
            }
        }
    }
}

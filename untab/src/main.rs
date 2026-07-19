use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    eprintln!("Usage: untab [OPTIONS] [FILE...]");
    eprintln!("Convert tabs to spaces.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -t TABSTOP   Tab stop positions (default: 8)");
    eprintln!("  -i           Only convert leading tabs");
    eprintln!("  -h, --help   Show this help message");
    eprintln!("  -v, --version Show version");
}

fn convert_line(line: &str, tabstop: usize, leading_only: bool) -> String {
    if leading_only {
        let mut result = String::new();
        let mut col = 0;
        let mut found_non_tab = false;
        for ch in line.chars() {
            if ch == '\t' && !found_non_tab {
                let spaces = tabstop - (col % tabstop);
                result.push_str(&" ".repeat(spaces));
                col += spaces;
            } else {
                if ch != '\t' {
                    found_non_tab = true;
                }
                result.push(ch);
                if ch == '\n' || ch == '\r' {
                    col = 0;
                } else {
                    col += 1;
                }
            }
        }
        result
    } else {
        let mut result = String::new();
        let mut col = 0;
        for ch in line.chars() {
            if ch == '\t' {
                let spaces = tabstop - (col % tabstop);
                result.push_str(&" ".repeat(spaces));
                col += spaces;
            } else {
                result.push(ch);
                if ch == '\n' || ch == '\r' {
                    col = 0;
                } else {
                    col += 1;
                }
            }
        }
        result
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut tabstop: usize = 8;
    let mut leading_only = false;
    let mut files = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-v" | "--version" => {
                println!("untab {VERSION}");
                return;
            }
            "-t" => {
                i += 1;
                if i < args.len() {
                    tabstop = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("untab: invalid tab stop '{}'", args[i]);
                        std::process::exit(1);
                    });
                    if tabstop == 0 {
                        eprintln!("untab: tab stop must be > 0");
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("untab: -t requires an argument");
                    std::process::exit(1);
                }
            }
            "-i" => leading_only = true,
            "--" => {
                i += 1;
                while i < args.len() {
                    files.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            _ if args[i].starts_with('-') => {
                eprintln!("untab: unknown option '{}'", args[i]);
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
            let converted = convert_line(&line, tabstop, leading_only);
            write!(out, "{converted}").ok();
        }
    } else {
        for path in &files {
            let file = File::open(path).unwrap_or_else(|e| {
                eprintln!("untab: cannot read '{path}': {e}");
                std::process::exit(1);
            });
            for line in BufReader::new(file).lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };
                let converted = convert_line(&line, tabstop, leading_only);
                write!(out, "{converted}").ok();
            }
        }
    }
}

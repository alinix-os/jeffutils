use std::env;
use std::fs;
use std::io::{self, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    eprintln!("Usage: meld [OPTIONS] FILE1 FILE2");
    eprintln!("Join lines of two files on a common field.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -a           Auto-join: treat each line as having the join field");
    eprintln!("  -e EMPTY     Fill missing fields with EMPTY string");
    eprintln!("  -j FIELD     Join on FIELD number (default: 1, 1-based)");
    eprintln!("  -t CHAR      Field separator character (default: tab)");
    eprintln!("  -h, --help   Show this help message");
    eprintln!("  -v, --version Show version");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_help();
        std::process::exit(1);
    }

    let mut auto_join = false;
    let mut empty = String::new();
    let mut join_field: usize = 1;
    let mut separator = '\t';
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-v" | "--version" => {
                println!("meld {VERSION}");
                return;
            }
            "-a" => auto_join = true,
            "-e" => {
                i += 1;
                if i < args.len() {
                    empty = args[i].clone();
                } else {
                    eprintln!("meld: option -e requires an argument");
                    std::process::exit(1);
                }
            }
            "-j" => {
                i += 1;
                if i < args.len() {
                    join_field = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("meld: invalid field number '{}'", args[i]);
                        std::process::exit(1);
                    });
                } else {
                    eprintln!("meld: option -j requires an argument");
                    std::process::exit(1);
                }
            }
            "-t" => {
                i += 1;
                if i < args.len() {
                    let s = &args[i];
                    separator = if s == "\\t" {
                        '\t'
                    } else if s == "\\n" {
                        '\n'
                    } else if s == "\\0" {
                        '\0'
                    } else {
                        s.chars().next().unwrap_or('\t')
                    };
                } else {
                    eprintln!("meld: option -t requires an argument");
                    std::process::exit(1);
                }
            }
            "--" => {
                i += 1;
                while i < args.len() {
                    files.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            _ if args[i].starts_with('-') => {
                eprintln!("meld: unknown option '{}'", args[i]);
                std::process::exit(1);
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    if files.len() < 2 {
        eprintln!("meld: two input files are required");
        std::process::exit(1);
    }

    let content1 = fs::read_to_string(&files[0]).unwrap_or_else(|e| {
        eprintln!("meld: cannot read '{}': {}", files[0], e);
        std::process::exit(1);
    });
    let content2 = fs::read_to_string(&files[1]).unwrap_or_else(|e| {
        eprintln!("meld: cannot read '{}': {}", files[1], e);
        std::process::exit(1);
    });

    let field_idx = if join_field == 0 { 0 } else { join_field - 1 };

    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for line in content1.lines() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(separator).collect();
        let key = if auto_join {
            line.to_string()
        } else if field_idx < fields.len() {
            fields[field_idx].to_string()
        } else {
            empty.clone()
        };
        map.entry(key).or_default().push(line.to_string());
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in content2.lines() {
        if line.is_empty() {
            continue;
        }
        let fields2: Vec<&str> = line.split(separator).collect();
        let key = if auto_join {
            line.to_string()
        } else if field_idx < fields2.len() {
            fields2[field_idx].to_string()
        } else {
            empty.clone()
        };

        if let Some(matches) = map.remove(&key) {
            for m in matches {
                writeln!(out, "{m}\t{line}").ok();
            }
        } else {
            let num_fields = fields2.len();
            let mut padding = String::new();
            for f in 0..join_field.saturating_sub(1) {
                if f > 0 {
                    padding.push(separator);
                }
                padding.push_str(&empty);
            }
            if join_field > 0 {
                if !padding.is_empty() || num_fields > 0 {
                    padding.push(separator);
                }
                padding.push_str(&key);
            }
            let rest: Vec<&str> = if field_idx < fields2.len() {
                fields2[(field_idx + 1)..].to_vec()
            } else {
                Vec::new()
            };
            if rest.is_empty() {
                writeln!(out, "{padding}").ok();
            } else {
                let suffix = rest.join(&separator.to_string());
                writeln!(out, "{padding}{separator}{suffix}").ok();
            }
        }
    }
}

use std::io::{self, BufRead, Write};
use std::process::exit;

const VERSION: &str = "0.1.0";

fn print_help() {
    eprintln!("arrange - sort lines of text");
    eprintln!("Usage: arrange [OPTIONS] [FILE...]");
    eprintln!("Read lines from files or stdin, sort them lexicographically.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -r        reverse sort order");
    eprintln!("  -n        numeric sort");
    eprintln!("  -u        unique lines (remove duplicates after sort)");
    eprintln!("  -h        print this help");
    eprintln!("  -v        print version");
}

fn parse_numeric(s: &str) -> f64 {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return 0.0;
    }
    trimmed.parse::<f64>().unwrap_or(0.0)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut reverse = false;
    let mut numeric = false;
    let mut unique = false;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-r" => reverse = true,
            "-n" => numeric = true,
            "-u" => unique = true,
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            "-v" | "--version" => {
                eprintln!("arrange {}", VERSION);
                exit(0);
            }
            "--" => {
                i += 1;
                while i < args.len() {
                    files.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            a if a.starts_with('-') && a.len() > 1 && !a.starts_with("--") => {
                for ch in a[1..].chars() {
                    match ch {
                        'r' => reverse = true,
                        'n' => numeric = true,
                        'u' => unique = true,
                        'h' => {
                            print_help();
                            exit(0);
                        }
                        'v' => {
                            eprintln!("arrange {}", VERSION);
                            exit(0);
                        }
                        _ => {
                            eprintln!("arrange: unknown option '-{}'", ch);
                            exit(1);
                        }
                    }
                }
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    let mut lines: Vec<String> = Vec::new();

    if files.is_empty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) => lines.push(l),
                Err(e) => {
                    eprintln!("arrange: read error: {}", e);
                    exit(1);
                }
            }
        }
    } else {
        for path in &files {
            let file = match std::fs::File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("arrange: {}: {}", path, e);
                    exit(1);
                }
            };
            let reader = io::BufReader::new(file);
            for line in reader.lines() {
                match line {
                    Ok(l) => lines.push(l),
                    Err(e) => {
                        eprintln!("arrange: {}: {}", path, e);
                        exit(1);
                    }
                }
            }
        }
    }

    if numeric {
        lines.sort_by(|a, b| {
            let na = parse_numeric(a);
            let nb = parse_numeric(b);
            na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        lines.sort();
    }

    if reverse {
        lines.reverse();
    }

    if unique {
        lines.dedup();
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in &lines {
        if writeln!(out, "{}", line).is_err() {
            eprintln!("arrange: write error");
            exit(1);
        }
    }
}

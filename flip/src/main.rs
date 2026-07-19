use std::io::{self, BufRead, Write};
use std::process::exit;

const VERSION: &str = "0.1.0";

fn print_help() {
    eprintln!("flip - reverse the order of lines");
    eprintln!("Usage: flip [OPTIONS] [FILE...]");
    eprintln!("Print lines of files in reverse order.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -s SEP   use SEP as the output separator (default: newline)");
    eprintln!("  -h       print this help");
    eprintln!("  -v       print version");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut separator = "\n".to_string();
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("flip: option -s requires an argument");
                    exit(1);
                }
                separator = args[i].clone();
            }
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            "-v" | "--version" => {
                eprintln!("flip {}", VERSION);
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
            a if a.starts_with('-') && a.len() > 1 => {
                for ch in a[1..].chars() {
                    match ch {
                        'h' => {
                            print_help();
                            exit(0);
                        }
                        'v' => {
                            eprintln!("flip {}", VERSION);
                            exit(0);
                        }
                        _ => {
                            eprintln!("flip: unknown option '-{}'", ch);
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
                    eprintln!("flip: read error: {}", e);
                    exit(1);
                }
            }
        }
    } else {
        for path in &files {
            let file = match std::fs::File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("flip: {}: {}", path, e);
                    exit(1);
                }
            };
            let reader = io::BufReader::new(file);
            for line in reader.lines() {
                match line {
                    Ok(l) => lines.push(l),
                    Err(e) => {
                        eprintln!("flip: {}: {}", path, e);
                        exit(1);
                    }
                }
            }
        }
    }

    lines.reverse();

    let stdout = io::stdout();
    let mut out = stdout.lock();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            if write!(out, "{}", separator).is_err() {
                eprintln!("flip: write error");
                exit(1);
            }
        }
        if write!(out, "{}", line).is_err() {
            eprintln!("flip: write error");
            exit(1);
        }
    }
}

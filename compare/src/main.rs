use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    eprintln!("Usage: compare [OPTIONS] FILE1 FILE2");
    eprintln!("Compare two sorted files line by line.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -1           Suppress column 1 (lines only in FILE1)");
    eprintln!("  -2           Suppress column 2 (lines only in FILE2)");
    eprintln!("  -3           Suppress column 3 (lines in both)");
    eprintln!("  -h, --help   Show this help message");
    eprintln!("  -v, --version Show version");
}

fn read_lines(path: &str) -> Vec<String> {
    let file = File::open(path).unwrap_or_else(|e| {
        eprintln!("compare: cannot read '{path}': {e}");
        std::process::exit(1);
    });
    BufReader::new(file)
        .lines()
        .map(|l| l.unwrap_or_default())
        .collect()
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_help();
        std::process::exit(1);
    }

    let mut suppress1 = false;
    let mut suppress2 = false;
    let mut suppress3 = false;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-v" | "--version" => {
                println!("compare {VERSION}");
                return;
            }
            "-1" => suppress1 = true,
            "-2" => suppress2 = true,
            "-3" => suppress3 = true,
            "--" => {
                i += 1;
                while i < args.len() {
                    files.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            _ if args[i].starts_with('-') => {
                eprintln!("compare: unknown option '{}'", args[i]);
                std::process::exit(1);
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    if files.len() < 2 {
        eprintln!("compare: two input files are required");
        std::process::exit(1);
    }

    let lines1 = read_lines(&files[0]);
    let lines2 = read_lines(&files[1]);

    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut i1 = 0;
    let mut i2 = 0;

    while i1 < lines1.len() && i2 < lines2.len() {
        match lines1[i1].cmp(&lines2[i2]) {
            std::cmp::Ordering::Less => {
                if !suppress1 {
                    writeln!(out, "{}", lines1[i1]).ok();
                }
                i1 += 1;
            }
            std::cmp::Ordering::Greater => {
                if !suppress2 {
                    writeln!(out, "{}", lines2[i2]).ok();
                }
                i2 += 1;
            }
            std::cmp::Ordering::Equal => {
                if !suppress3 {
                    writeln!(out, "{}", lines1[i1]).ok();
                }
                i1 += 1;
                i2 += 1;
            }
        }
    }

    while i1 < lines1.len() {
        if !suppress1 {
            writeln!(out, "{}", lines1[i1]).ok();
        }
        i1 += 1;
    }

    while i2 < lines2.len() {
        if !suppress2 {
            writeln!(out, "{}", lines2[i2]).ok();
        }
        i2 += 1;
    }
}

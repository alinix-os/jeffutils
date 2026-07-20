use std::env;
use std::io::{self, BufRead, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    eprintln!("Usage: primegen [NUMBER...]");
    eprintln!("Factorize numbers into prime factors.");
    eprintln!();
    eprintln!("Numbers can be provided as arguments or read from stdin (one per line).");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help    Show this help message");
    eprintln!("  -v, --version Show version");
}

fn factorize(mut n: u64) -> Vec<u64> {
    if n <= 1 {
        return vec![n];
    }
    let mut factors = Vec::new();
    let mut d = 2u64;
    while d * d <= n {
        while n % d == 0 {
            factors.push(d);
            n /= d;
        }
        d += 1;
    }
    if n > 1 {
        factors.push(n);
    }
    factors
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("primegen", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("primegen {VERSION}");
        return;
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    if args.is_empty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let n: u64 = match trimmed.parse() {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("primegen: invalid number '{trimmed}'");
                    continue;
                }
            };
            let factors = factorize(n);
            let display: Vec<String> = factors.iter().map(|f| f.to_string()).collect();
            if factors.len() == 1 {
                writeln!(out, "{n} = {}", display[0]).ok();
            } else {
                writeln!(out, "{n} = {}", display.join(" * ")).ok();
            }
        }
    } else {
        for arg in &args {
            let n: u64 = match arg.parse() {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("primegen: invalid number '{arg}'");
                    continue;
                }
            };
            let factors = factorize(n);
            let display: Vec<String> = factors.iter().map(|f| f.to_string()).collect();
            if factors.len() == 1 {
                writeln!(out, "{n} = {}", display[0]).ok();
            } else {
                writeln!(out, "{n} = {}", display.join(" * ")).ok();
            }
        }
    }
}

use std::io::{self, BufRead, Write};
use std::process::exit;

const VERSION: &str = "0.1.0";

fn print_help() {
    eprintln!("number - number lines of text");
    eprintln!("Usage: number [OPTIONS] [FILE...]");
    eprintln!("Number lines of files or stdin.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -b STYLE   body numbering style: a=all (default), t=nonempty, n=never");
    eprintln!("  -s SEP     separator after line number (default: tab)");
    eprintln!("  -w WIDTH   width of line numbers (default: 6)");
    eprintln!("  -h         print this help");
    eprintln!("  -v         print version");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut body_style = "a".to_string();
    let mut separator = "\t".to_string();
    let mut width: usize = 6;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-b" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("number: option -b requires an argument");
                    exit(1);
                }
                body_style = args[i].clone();
            }
            "-s" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("number: option -s requires an argument");
                    exit(1);
                }
                separator = args[i].clone();
            }
            "-w" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("number: option -w requires an argument");
                    exit(1);
                }
                width = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("number: invalid width: {}", args[i]);
                    exit(1);
                });
            }
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            "-v" | "--version" => {
                eprintln!("number {}", VERSION);
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
                let chars: Vec<char> = a[1..].chars().collect();
                let mut j = 0;
                while j < chars.len() {
                    match chars[j] {
                        'b' => {
                            j += 1;
                            if j < chars.len() {
                                body_style = chars[j].to_string();
                            } else {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("number: option -b requires an argument");
                                    exit(1);
                                }
                                body_style = args[i].clone();
                            }
                        }
                        's' => {
                            j += 1;
                            if j < chars.len() {
                                separator = chars[j].to_string();
                            } else {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("number: option -s requires an argument");
                                    exit(1);
                                }
                                separator = args[i].clone();
                            }
                        }
                        'w' => {
                            j += 1;
                            if j < chars.len() {
                                let rest: String = chars[j..].iter().collect();
                                width = rest.parse().unwrap_or_else(|_| {
                                    eprintln!("number: invalid width: {}", rest);
                                    exit(1);
                                });
                                break;
                            } else {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("number: option -w requires an argument");
                                    exit(1);
                                }
                                width = args[i].parse().unwrap_or_else(|_| {
                                    eprintln!("number: invalid width: {}", args[i]);
                                    exit(1);
                                });
                            }
                        }
                        'h' => {
                            print_help();
                            exit(0);
                        }
                        'v' => {
                            eprintln!("number {}", VERSION);
                            exit(0);
                        }
                        _ => {
                            eprintln!("number: unknown option '-{}'", chars[j]);
                            exit(1);
                        }
                    }
                    j += 1;
                }
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    let should_number = |line: &str| -> bool {
        match body_style.as_str() {
            "a" => true,
            "t" => !line.trim().is_empty(),
            "n" => false,
            _ => true,
        }
    };

    let mut line_num: usize = 1;

    let mut process_line = |line: &str| {
        let stdout = io::stdout();
        let mut out = stdout.lock();

        if should_number(line) {
            writeln!(out, "{:>width$}{}{}", line_num, separator, line, width = width).unwrap();
            line_num += 1;
        } else {
            writeln!(out, "{}", line).unwrap();
        }
    };

    if files.is_empty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) => process_line(&l),
                Err(e) => {
                    eprintln!("number: read error: {}", e);
                    exit(1);
                }
            }
        }
    } else {
        for path in &files {
            let file = match std::fs::File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("number: {}: {}", path, e);
                    exit(1);
                }
            };
            let reader = io::BufReader::new(file);
            for line in reader.lines() {
                match line {
                    Ok(l) => process_line(&l),
                    Err(e) => {
                        eprintln!("number: {}: {}", path, e);
                        exit(1);
                    }
                }
            }
        }
    }
}

use std::env;
use std::io::{self, BufRead, BufReader, Write};

fn print_usage() {
    eprintln!("Usage: wrap [OPTION]... [FILE]...");
    eprintln!("Wrap input lines at WIDTH columns.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -s           break at spaces (word boundaries)");
    eprintln!("  -w WIDTH     wrap at WIDTH columns (default 80)");
    eprintln!("  -h, --help       display this help and exit");
    eprintln!("  -v, --version    display version and exit");
}

fn wrap_line(line: &str, width: usize, break_at_space: bool) -> Vec<String> {
    if line.len() <= width {
        return vec![line.to_string()];
    }

    let mut result = Vec::new();
    let mut remaining = line;

    while remaining.len() > width {
        let break_at = if break_at_space {
            remaining[..width]
                .rfind(' ')
                .unwrap_or(width)
        } else {
            width
        };

        result.push(remaining[..break_at].to_string());
        remaining = &remaining[break_at..];
        remaining = remaining.trim_start_matches(' ');
    }

    if !remaining.is_empty() {
        result.push(remaining.to_string());
    }

    result
}

fn wrap_reader(reader: &mut dyn BufRead, width: usize, break_at_space: bool) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut buffer = String::new();
    loop {
        buffer.clear();
        match reader.read_line(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = buffer.trim_end_matches('\n');
                let trimmed = trimmed.trim_end_matches('\r');
                for wrapped in wrap_line(trimmed, width, break_at_space) {
                    writeln!(out, "{}", wrapped)?;
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("wrap (JeffUtils) 1.0");
        return;
    }

    let mut width = 80;
    let mut break_at_space = false;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" => break_at_space = true,
            "-w" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("wrap: option '-w' requires an argument");
                    std::process::exit(1);
                }
                width = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("wrap: invalid width '{}'", args[i]);
                    std::process::exit(1);
                });
            }
            other if other.starts_with("-w=") => {
                width = other[3..].parse().unwrap_or_else(|_| {
                    eprintln!("wrap: invalid width '{}'", &other[3..]);
                    std::process::exit(1);
                });
            }
            "--version" => {}
            "--help" => {}
            other if other.starts_with('-') && other.len() > 1 => {
                for ch in other[1..].chars() {
                    match ch {
                        's' => break_at_space = true,
                        'w' => {
                            eprintln!("wrap: option '-w' requires an argument");
                            std::process::exit(1);
                        }
                        _ => {
                            eprintln!("wrap: unknown option '-{}'", ch);
                            std::process::exit(1);
                        }
                    }
                }
            }
            _ => {
                files.push(args[i].clone());
            }
        }
        i += 1;
    }

    if files.is_empty() {
        let mut stdin = BufReader::new(io::stdin());
        if let Err(e) = wrap_reader(&mut stdin, width, break_at_space) {
            eprintln!("wrap: {}", e);
            std::process::exit(1);
        }
    } else {
        for file in &files {
            match std::fs::File::open(file) {
                Ok(f) => {
                    let mut reader = io::BufReader::new(f);
                    if let Err(e) = wrap_reader(&mut reader, width, break_at_space) {
                        eprintln!("wrap: {}: {}", file, e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("wrap: {}: {}", file, e);
                    std::process::exit(1);
                }
            }
        }
    }
}

use std::io::{self, BufRead, Write};
use std::process::exit;

const VERSION: &str = "0.1.0";

fn print_help() {
    eprintln!("stitch - merge lines of files side by side");
    eprintln!("Usage: stitch [OPTIONS] FILE...");
    eprintln!("Merge lines of files side by side.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -d DELIM   delimiter between columns (default: tab)");
    eprintln!("  -s         serial mode: paste one file at a time, then the next");
    eprintln!("  -h         print this help");
    eprintln!("  -v         print version");
}

fn read_file_lines(path: &str) -> Vec<String> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("stitch: {}: {}", path, e);
            exit(1);
        }
    };
    let reader = io::BufReader::new(file);
    reader.lines().filter_map(|l| l.ok()).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut delimiter = "\t".to_string();
    let mut serial_mode = false;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("stitch: option -d requires an argument");
                    exit(1);
                }
                delimiter = args[i].clone();
            }
            "-s" => serial_mode = true,
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            "-v" | "--version" => {
                eprintln!("stitch {}", VERSION);
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
                        's' => serial_mode = true,
                        'h' => {
                            print_help();
                            exit(0);
                        }
                        'v' => {
                            eprintln!("stitch {}", VERSION);
                            exit(0);
                        }
                        _ => {
                            eprintln!("stitch: unknown option '-{}'", ch);
                            exit(1);
                        }
                    }
                }
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    if files.is_empty() {
        eprintln!("stitch: missing file operand");
        eprintln!("Try 'stitch -h' for more information.");
        exit(1);
    }

    let all_lines: Vec<Vec<String>> = files.iter().map(|f| read_file_lines(f)).collect();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    if serial_mode {
        for file_lines in &all_lines {
            for line in file_lines {
                writeln!(out, "{}", line).unwrap();
            }
        }
    } else {
        let max_lines = all_lines.iter().map(|l| l.len()).max().unwrap_or(0);
        for row in 0..max_lines {
            let mut first = true;
            for file_lines in &all_lines {
                if !first {
                    write!(out, "{}", delimiter).unwrap();
                }
                if row < file_lines.len() {
                    write!(out, "{}", file_lines[row]).unwrap();
                }
                first = false;
            }
            writeln!(out).unwrap();
        }
    }
}

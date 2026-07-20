use std::env;
use std::fs;
use std::io::{self, Read, Write, BufWriter};
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("permutext - generate permuted index");
    eprintln!();
    eprintln!("USAGE: permutext [FILE...]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -w WIDTH    output width (default: 72)");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
}

fn process_text(text: &str, width: usize) {
    let lines: Vec<&str> = text.lines().collect();
    let mut entries: Vec<(String, usize, usize)> = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        let words: Vec<&str> = line.split_whitespace().collect();
        for (word_idx, word) in words.iter().enumerate() {
            let key = word.to_lowercase();
            entries.push((key, line_idx, word_idx));
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let key_width = width / 3;
    let ref_width = width - key_width - 2;

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    for (key, line_idx, word_idx) in &entries {
        let line = &lines[*line_idx];
        let words: Vec<&str> = line.split_whitespace().collect();

        let mut permuted_line = String::new();
        for i in *word_idx..words.len() {
            if !permuted_line.is_empty() {
                permuted_line.push(' ');
            }
            permuted_line.push_str(words[i]);
        }
        for i in 0..*word_idx {
            if !permuted_line.is_empty() {
                permuted_line.push(' ');
            }
            permuted_line.push_str(words[i]);
        }

        if permuted_line.len() > ref_width {
            permuted_line.truncate(ref_width - 3);
            permuted_line.push_str("...");
        }

        let padded_key = if key.len() > key_width {
            key[..key_width - 1].to_string() + "*"
        } else {
            format!("{:>width$}", key, width = key_width)
        };

        let padded_ref = if permuted_line.len() > ref_width {
            permuted_line[..ref_width].to_string()
        } else {
            format!("{:<width$}", permuted_line, width = ref_width)
        };

        let _ = writeln!(out, "{}  {}", padded_key, padded_ref);
    }
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("permutext", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    let mut width: usize = 72;
    let mut files: Vec<String> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("permutext {}", VERSION);
                process::exit(0);
            }
            "-w" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("permutext: option -w requires an argument");
                    process::exit(2);
                }
                width = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("permutext: invalid width '{}'", args[i]);
                    process::exit(2);
                });
            }
            _ if args[i].starts_with('-') => {
                eprintln!("permutext: unknown option '{}'", args[i]);
                process::exit(2);
            }
            _ => {
                files.push(args[i].clone());
            }
        }
        i += 1;
    }

    if files.is_empty() {
        let mut text = String::new();
        io::stdin().read_to_string(&mut text).unwrap_or_else(|e| {
            eprintln!("permutext: error reading stdin: {}", e);
            process::exit(1);
        });
        process_text(&text, width);
    } else {
        for file in &files {
            let text = fs::read_to_string(file).unwrap_or_else(|e| {
                eprintln!("permutext: cannot open '{}': {}", file, e);
                process::exit(1);
            });
            process_text(&text, width);
        }
    }
}

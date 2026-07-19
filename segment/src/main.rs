use std::env;
use std::fs;
use std::io::{self, BufRead, Write, BufWriter};
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("segment - split file by pattern");
    eprintln!();
    eprintln!("USAGE: segment FILE PATTERN...");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -f FORMAT   output file prefix (default: xx)");
    eprintln!("  -n NUM      digits in suffix (default: 2)");
    eprintln!("  -z          remove empty output files");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        process::exit(2);
    }

    let mut prefix = String::from("xx");
    let mut suffix_digits: usize = 2;
    let mut remove_empty = false;
    let mut positional: Vec<String> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("segment {}", VERSION);
                process::exit(0);
            }
            "-f" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("segment: option -f requires an argument");
                    process::exit(2);
                }
                prefix = args[i].clone();
            }
            "-n" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("segment: option -n requires an argument");
                    process::exit(2);
                }
                suffix_digits = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("segment: invalid number '{}'", args[i]);
                    process::exit(2);
                });
            }
            "-z" => {
                remove_empty = true;
            }
            _ if args[i].starts_with('-') => {
                eprintln!("segment: unknown option '{}'", args[i]);
                process::exit(2);
            }
            _ => {
                positional.push(args[i].clone());
            }
        }
        i += 1;
    }

    if positional.is_empty() {
        eprintln!("segment: missing FILE");
        print_usage();
        process::exit(2);
    }

    let filename = &positional[0];
    let patterns: Vec<&str> = positional[1..].iter().map(|s| s.as_str()).collect();

    if patterns.is_empty() {
        eprintln!("segment: missing PATTERN");
        print_usage();
        process::exit(2);
    }

    let file = fs::File::open(filename).unwrap_or_else(|e| {
        eprintln!("segment: cannot open '{}': {}", filename, e);
        process::exit(1);
    });

    let reader = io::BufReader::new(file);
    let mut part_num: u64 = 0;
    let mut lines: Vec<String> = Vec::new();
    let mut total_written: u64 = 0;

    fn write_part(prefix: &str, suffix_digits: usize, part_num: u64, lines: &[String], remove_empty: bool) -> io::Result<bool> {
        if remove_empty && lines.is_empty() {
            return Ok(false);
        }
        let suffix = format!("{:0width$}", part_num, width = suffix_digits);
        let out_name = format!("{}{}", prefix, suffix);
        let mut out = BufWriter::new(fs::File::create(&out_name)?);
        for line in lines {
            writeln!(out, "{}", line)?;
        }
        Ok(true)
    }

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("segment: read error: {}", e);
                process::exit(1);
            }
        };

        let matches = patterns.iter().any(|pat| line.contains(pat));

        if matches && !lines.is_empty() {
            if write_part(&prefix, suffix_digits, part_num, &lines, remove_empty).unwrap_or_else(|e| {
                eprintln!("segment: write error: {}", e);
                process::exit(1);
            }) {
                total_written += 1;
            }
            part_num += 1;
            lines.clear();
        }

        if matches {
            lines.push(line.clone());
        } else {
            lines.push(line);
        }
    }

    if write_part(&prefix, suffix_digits, part_num, &lines, remove_empty).unwrap_or_else(|e| {
        eprintln!("segment: write error: {}", e);
        process::exit(1);
    }) {
        total_written += 1;
    }

    eprintln!("segment: created {} parts", total_written);
}

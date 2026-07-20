use std::io::{self, Read, Write};
use std::process::exit;

const VERSION: &str = "0.1.0";

fn print_help() {
    eprintln!("chunk - split a file into pieces");
    eprintln!("Usage: chunk [OPTIONS] FILE");
    eprintln!("Split a file into multiple smaller files.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -b SIZE    split by SIZE bytes (supports K, M, G suffixes)");
    eprintln!("  -l LINES   split by LINES lines");
    eprintln!("  -d         use numeric suffixes (aa, ab, ...)");
    eprintln!("  -a LENGTH  suffix length (default: 2)");
    eprintln!("  -p PATTERN break at pattern (use \\0 for NUL)");
    eprintln!("  -h         print this help");
    eprintln!("  -v         print version");
    eprintln!();
    eprintln!("Output files are named: FILEaa, FILEab, ... (or FILE00, FILE01, ... with -d)");
}

fn parse_size(s: &str) -> Option<usize> {
    let s = s.trim();
    let (num_str, multiplier) = if let Some(pos) = s.find(|c: char| c.is_alphabetic()) {
        let (num, suffix) = s.split_at(pos);
        let m = match suffix.to_lowercase().as_str() {
            "k" => 1024,
            "m" => 1024 * 1024,
            "g" => 1024 * 1024 * 1024,
            _ => {
                eprintln!("chunk: invalid size suffix: {}", suffix);
                return None;
            }
        };
            (num, m)
        } else {
            (s, 1)
        };

    let n: usize = num_str.parse().ok()?;
    Some(n * multiplier)
}

fn make_suffix(index: usize, length: usize, numeric: bool) -> String {
    if numeric {
        format!("{:0width$}", index, width = length)
    } else {
        let mut result = String::new();
        let mut idx = index;
        for _ in 0..length {
            result.push((b'a' + (idx % 26) as u8) as char);
            idx /= 26;
        }
        result.chars().rev().collect()
    }
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("chunk", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut bytes_per_chunk: Option<usize> = None;
    let mut lines_per_chunk: Option<usize> = None;
    let mut numeric_suffix = false;
    let mut suffix_length: usize = 2;
    let mut pattern: Option<String> = None;
    let mut file_arg: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-b" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("chunk: option -b requires an argument");
                    exit(1);
                }
                bytes_per_chunk = Some(parse_size(&args[i]).unwrap_or_else(|| {
                    eprintln!("chunk: invalid size: {}", args[i]);
                    exit(1);
                }));
            }
            "-l" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("chunk: option -l requires an argument");
                    exit(1);
                }
                lines_per_chunk = Some(args[i].parse().unwrap_or_else(|_| {
                    eprintln!("chunk: invalid number: {}", args[i]);
                    exit(1);
                }));
            }
            "-d" => numeric_suffix = true,
            "-a" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("chunk: option -a requires an argument");
                    exit(1);
                }
                suffix_length = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("chunk: invalid suffix length: {}", args[i]);
                    exit(1);
                });
            }
            "-p" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("chunk: option -p requires an argument");
                    exit(1);
                }
                let p = args[i].replace("\\0", "\0");
                pattern = Some(p);
            }
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            "-v" | "--version" => {
                eprintln!("chunk {}", VERSION);
                exit(0);
            }
            "--" => {
                i += 1;
                if i < args.len() {
                    file_arg = Some(args[i].clone());
                }
                break;
            }
            a if a.starts_with('-') && a.len() > 1 => {
                eprintln!("chunk: unknown option '{}'", a);
                exit(1);
            }
            _ => {
                if file_arg.is_none() {
                    file_arg = Some(args[i].clone());
                } else {
                    eprintln!("chunk: unexpected argument: {}", args[i]);
                    exit(1);
                }
            }
        }
        i += 1;
    }

    let filename = match file_arg {
        Some(f) => f,
        None => {
            eprintln!("chunk: missing file operand");
            eprintln!("Try 'chunk -h' for more information.");
            exit(1);
        }
    };

    let mut input = Vec::new();
    let file = match std::fs::File::open(&filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("chunk: {}: {}", filename, e);
            exit(1);
        }
    };
    let mut reader = io::BufReader::new(file);
    reader.read_to_end(&mut input).unwrap_or_else(|e| {
        eprintln!("chunk: read error: {}", e);
        exit(1);
    });

    let mut part_index: usize = 0;

    if let Some(size) = bytes_per_chunk {
        let mut offset = 0;
        while offset < input.len() {
            let end = (offset + size).min(input.len());
            let chunk = &input[offset..end];
            let suffix = make_suffix(part_index, suffix_length, numeric_suffix);
            let out_name = format!("{}{}", filename, suffix);
            let mut out_file = std::fs::File::create(&out_name).unwrap_or_else(|e| {
                eprintln!("chunk: {}: {}", out_name, e);
                exit(1);
            });
            out_file.write_all(chunk).unwrap_or_else(|e| {
                eprintln!("chunk: write error: {}", e);
                exit(1);
            });
            offset = end;
            part_index += 1;
        }
    } else if let Some(max_lines) = lines_per_chunk {
        let text = String::from_utf8_lossy(&input);
        let mut lines: Vec<&str> = text.split('\n').collect();
        if lines.last() == Some(&"") {
            lines.pop();
        }

        let mut chunk_start = 0;
        while chunk_start < lines.len() {
            let chunk_end = (chunk_start + max_lines).min(lines.len());
            let suffix = make_suffix(part_index, suffix_length, numeric_suffix);
            let out_name = format!("{}{}", filename, suffix);
            let mut out_file = std::fs::File::create(&out_name).unwrap_or_else(|e| {
                eprintln!("chunk: {}: {}", out_name, e);
                exit(1);
            });
            for line in &lines[chunk_start..chunk_end] {
                writeln!(out_file, "{}", line).unwrap_or_else(|e| {
                    eprintln!("chunk: write error: {}", e);
                    exit(1);
                });
            }
            chunk_start = chunk_end;
            part_index += 1;
        }
    } else if let Some(ref pat) = pattern {
        let mut offset = 0;
        while offset < input.len() {
            let search_start = if part_index > 0 { offset } else { offset };
            let remaining = &input[search_start..];
            let break_at = remaining.windows(pat.len()).position(|w| w == pat.as_bytes());
            let end = match break_at {
                Some(pos) => search_start + pos + pat.len(),
                None => input.len(),
            };
            let chunk = &input[offset..end];
            let suffix = make_suffix(part_index, suffix_length, numeric_suffix);
            let out_name = format!("{}{}", filename, suffix);
            let mut out_file = std::fs::File::create(&out_name).unwrap_or_else(|e| {
                eprintln!("chunk: {}: {}", out_name, e);
                exit(1);
            });
            out_file.write_all(chunk).unwrap_or_else(|e| {
                eprintln!("chunk: write error: {}", e);
                exit(1);
            });
            offset = end;
            part_index += 1;
            if break_at.is_none() {
                break;
            }
        }
    } else {
        eprintln!("chunk: specify -b, -l, or -p");
        exit(1);
    }

    eprintln!("chunk: created {} parts from {}", part_index, filename);
}

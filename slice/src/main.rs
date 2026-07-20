use std::io::{self, BufRead, Write};
use std::process::exit;

const VERSION: &str = "0.1.0";

fn print_help() {
    eprintln!("slice - cut fields from lines");
    eprintln!("Usage: slice [OPTIONS] [FILE...]");
    eprintln!("Cut fields from each line of files or stdin.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -d DELIM   field delimiter (default: tab)");
    eprintln!("  -f LIST    field numbers to extract (1-based, comma-separated, ranges like 1-3,5)");
    eprintln!("  -c CHARS   character positions to extract (1-based, same range syntax as -f)");
    eprintln!("  -h         print this help");
    eprintln!("  -v         print version");
}

fn parse_range_list(list: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    for part in list.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((start_s, end_s)) = part.split_once('-') {
            let start: usize = start_s.trim().parse().unwrap_or_else(|_| {
                eprintln!("slice: invalid range: {}", part);
                exit(1);
            });
            let end: usize = end_s.trim().parse().unwrap_or_else(|_| {
                eprintln!("slice: invalid range: {}", part);
                exit(1);
            });
            ranges.push((start, end));
        } else {
            let n: usize = part.parse().unwrap_or_else(|_| {
                eprintln!("slice: invalid number: {}", part);
                exit(1);
            });
            ranges.push((n, n));
        }
    }
    ranges
}

fn ranges_contain(ranges: &[(usize, usize)], pos: usize) -> bool {
    for &(start, end) in ranges {
        if pos >= start && pos <= end {
            return true;
        }
    }
    false
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("slice", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut delimiter: String = "\t".to_string();
    let mut field_list: Option<String> = None;
    let mut char_list: Option<String> = None;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("slice: option -d requires an argument");
                    exit(1);
                }
                delimiter = args[i].clone();
            }
            "-f" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("slice: option -f requires an argument");
                    exit(1);
                }
                field_list = Some(args[i].clone());
            }
            "-c" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("slice: option -c requires an argument");
                    exit(1);
                }
                char_list = Some(args[i].clone());
            }
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            "-v" | "--version" => {
                eprintln!("slice {}", VERSION);
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
                        'd' => {
                            j += 1;
                            if j < chars.len() {
                                delimiter = chars[j].to_string();
                            } else {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("slice: option -d requires an argument");
                                    exit(1);
                                }
                                delimiter = args[i].clone();
                            }
                        }
                        'f' => {
                            j += 1;
                            if j < chars.len() {
                                let rest: String = chars[j..].iter().collect();
                                field_list = Some(rest);
                                break;
                            } else {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("slice: option -f requires an argument");
                                    exit(1);
                                }
                                field_list = Some(args[i].clone());
                            }
                        }
                        'c' => {
                            j += 1;
                            if j < chars.len() {
                                let rest: String = chars[j..].iter().collect();
                                char_list = Some(rest);
                                break;
                            } else {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("slice: option -c requires an argument");
                                    exit(1);
                                }
                                char_list = Some(args[i].clone());
                            }
                        }
                        'h' => {
                            print_help();
                            exit(0);
                        }
                        'v' => {
                            eprintln!("slice {}", VERSION);
                            exit(0);
                        }
                        _ => {
                            eprintln!("slice: unknown option '-{}'", chars[j]);
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

    let ranges = if let Some(ref fl) = field_list {
        parse_range_list(fl)
    } else {
        Vec::new()
    };

    let char_ranges = if let Some(ref cl) = char_list {
        parse_range_list(cl)
    } else {
        Vec::new()
    };

    let use_char_mode = !char_ranges.is_empty();
    let use_field_mode = !ranges.is_empty();

    let process_line = |line: &str| {
        let stdout = io::stdout();
        let mut out = stdout.lock();

        if use_char_mode {
            let chars: Vec<char> = line.chars().collect();
            let mut result = String::new();
            for (idx, ch) in chars.iter().enumerate() {
                let pos = idx + 1;
                if ranges_contain(&char_ranges, pos) {
                    result.push(*ch);
                }
            }
            writeln!(out, "{}", result).unwrap();
        } else if use_field_mode {
            let fields: Vec<&str> = line.split(&*delimiter).collect();
            let mut first = true;
            for &(start, end) in &ranges {
                for idx in start..=end {
                    if idx - 1 < fields.len() {
                        if !first {
                            write!(out, "{}", delimiter).unwrap();
                        }
                        write!(out, "{}", fields[idx - 1]).unwrap();
                        first = false;
                    }
                }
            }
            writeln!(out).unwrap();
        } else {
            writeln!(out, "{}", line).unwrap();
        }
    };

    let process_reader = |reader: Box<dyn io::BufRead>| {
        for line_result in reader.lines() {
            match line_result {
                Ok(line) => process_line(&line),
                Err(e) => {
                    eprintln!("slice: read error: {}", e);
                    exit(1);
                }
            }
        }
    };

    if files.is_empty() {
        let stdin = io::stdin();
        process_reader(Box::new(stdin.lock()));
    } else {
        for path in &files {
            let file = match std::fs::File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("slice: {}: {}", path, e);
                    exit(1);
                }
            };
            process_reader(Box::new(io::BufReader::new(file)));
        }
    }
}

use std::io::{self, BufRead, Write};
use std::process::exit;

const VERSION: &str = "0.1.0";

fn print_help() {
    eprintln!("dedup - filter adjacent duplicate lines");
    eprintln!("Usage: dedup [OPTIONS] [FILE...]");
    eprintln!("Read lines and print only adjacent duplicate or unique lines.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -c        prefix lines by the number of occurrences");
    eprintln!("  -d        only print duplicate lines");
    eprintln!("  -u        only print unique lines");
    eprintln!("  -i        ignore differences in case when comparing");
    eprintln!("  -s NUM    skip first NUM characters of each line");
    eprintln!("  -w NUM    compare at most NUM characters of each line");
    eprintln!("  -h        print this help");
    eprintln!("  -v        print version");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut count_mode = false;
    let mut dup_only = false;
    let mut uniq_only = false;
    let mut case_insensitive = false;
    let mut skip_chars: usize = 0;
    let mut compare_chars: Option<usize> = None;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-c" => count_mode = true,
            "-d" => dup_only = true,
            "-u" => uniq_only = true,
            "-i" => case_insensitive = true,
            "-s" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("dedup: option -s requires an argument");
                    exit(1);
                }
                skip_chars = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("dedup: invalid number for -s: {}", args[i]);
                    exit(1);
                });
            }
            "-w" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("dedup: option -w requires an argument");
                    exit(1);
                }
                compare_chars = Some(args[i].parse().unwrap_or_else(|_| {
                    eprintln!("dedup: invalid number for -w: {}", args[i]);
                    exit(1);
                }));
            }
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            "-v" | "--version" => {
                eprintln!("dedup {}", VERSION);
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
                        'c' => count_mode = true,
                        'd' => dup_only = true,
                        'u' => uniq_only = true,
                        'i' => case_insensitive = true,
                        'h' => {
                            print_help();
                            exit(0);
                        }
                        'v' => {
                            eprintln!("dedup {}", VERSION);
                            exit(0);
                        }
                        _ => {
                            eprintln!("dedup: unknown option '-{}'", ch);
                            exit(1);
                        }
                    }
                }
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    fn compare_key(line: &str, skip: usize, max_len: Option<usize>, ci: bool) -> String {
        let s: String = line.chars().skip(skip).collect();
        let s = if let Some(m) = max_len {
            s.chars().take(m).collect()
        } else {
            s
        };
        if ci {
            s.to_lowercase()
        } else {
            s
        }
    }

    let process_lines = |lines: Vec<String>| {
        let stdout = io::stdout();
        let mut out = stdout.lock();

        if lines.is_empty() {
            return;
        }

        let mut groups: Vec<(String, usize)> = Vec::new();
        let key = compare_key(&lines[0], skip_chars, compare_chars, case_insensitive);
        groups.push((key, 1));
        let mut line_groups: Vec<Vec<&String>> = vec![vec![&lines[0]]];

        for line in lines.iter().skip(1) {
            let key = compare_key(line, skip_chars, compare_chars, case_insensitive);
            if let Some(last) = groups.last_mut() {
                if last.0 == key {
                    last.1 += 1;
                    line_groups.last_mut().unwrap().push(line);
                    continue;
                }
            }
            groups.push((key, 1));
            line_groups.push(vec![line]);
        }

        for (group, lines_in_group) in line_groups.iter().enumerate() {
            let count = groups[group].1;
            let is_dup = count > 1;

            if dup_only && !is_dup {
                continue;
            }
            if uniq_only && is_dup {
                continue;
            }

            if count_mode {
                if writeln!(out, "{:>7} {}", count, lines_in_group[0]).is_err() {
                    eprintln!("dedup: write error");
                    exit(1);
                }
            } else {
                for line in lines_in_group {
                    if writeln!(out, "{}", line).is_err() {
                        eprintln!("dedup: write error");
                        exit(1);
                    }
                }
            }
        }
    };

    if files.is_empty() {
        let stdin = io::stdin();
        let lines: Vec<String> = stdin.lock().lines().filter_map(|l| l.ok()).collect();
        process_lines(lines);
    } else {
        for path in &files {
            let file = match std::fs::File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("dedup: {}: {}", path, e);
                    exit(1);
                }
            };
            let reader = io::BufReader::new(file);
            let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
            process_lines(lines);
        }
    }
}

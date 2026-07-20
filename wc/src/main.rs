use std::env;
use std::fs::File;
use std::io::{self, Read, BufReader};

fn print_usage() {
    eprintln!("Usage: wc [OPTION]... [FILE]...");
    eprintln!("Print newline, word, and byte counts for each FILE, and a total line if");
    eprintln!("more than one FILE is specified.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -c, --bytes      print the byte counts");
    eprintln!("  -m, --chars      print the character counts");
    eprintln!("  -l, --lines      print the newline counts");
    eprintln!("  -w, --words      print the word counts");
    eprintln!("  -h, --help       display this help and exit");
    eprintln!("      --version    output version information and exit");
}

struct Counts {
    lines: usize,
    words: usize,
    bytes: usize,
    chars: usize,
}

fn count_reader<R: Read>(mut reader: R) -> io::Result<Counts> {
    let mut lines = 0;
    let mut words = 0;
    let mut bytes = 0;
    let mut chars = 0;
    let mut in_word = false;

    let mut leftover = Vec::new();
    loop {
        let mut read_buf = [0; 8192];
        let n = reader.read(&mut read_buf)?;
        if n == 0 {
            break;
        }
        bytes += n;

        let mut chunk_bytes = &read_buf[..n];
        let combined;
        if !leftover.is_empty() {
            leftover.extend_from_slice(chunk_bytes);
            combined = leftover.clone();
            chunk_bytes = &combined;
            leftover.clear();
        }

        match std::str::from_utf8(chunk_bytes) {
            Ok(s) => {
                for c in s.chars() {
                    chars += 1;
                    if c == '\n' {
                        lines += 1;
                    }
                    if c.is_whitespace() {
                        in_word = false;
                    } else if !in_word {
                        in_word = true;
                        words += 1;
                    }
                }
            }
            Err(e) => {
                let valid_len = e.valid_up_to();
                let valid_str = std::str::from_utf8(&chunk_bytes[..valid_len]).unwrap_or("");
                for c in valid_str.chars() {
                    chars += 1;
                    if c == '\n' {
                        lines += 1;
                    }
                    if c.is_whitespace() {
                        in_word = false;
                    } else if !in_word {
                        in_word = true;
                        words += 1;
                    }
                }

                if let Some(error_len) = e.error_len() {
                    let next_start = valid_len + error_len;
                    leftover.extend_from_slice(&chunk_bytes[next_start..]);
                } else {
                    leftover.extend_from_slice(&chunk_bytes[valid_len..]);
                }
            }
        }
    }

    if !leftover.is_empty() {
        let s = String::from_utf8_lossy(&leftover);
        for c in s.chars() {
            chars += 1;
            if c == '\n' {
                lines += 1;
            }
            if c.is_whitespace() {
                in_word = false;
            } else if !in_word {
                in_word = true;
                words += 1;
            }
        }
    }

    Ok(Counts { lines, words, bytes, chars })
}

fn print_counts(counts: &Counts, show_lines: bool, show_words: bool, show_bytes: bool, show_chars: bool, label: &str) {
    let mut outputs = Vec::new();
    if show_lines {
        outputs.push(format!("{:>7}", counts.lines));
    }
    if show_words {
        outputs.push(format!("{:>7}", counts.words));
    }
    if show_chars {
        outputs.push(format!("{:>7}", counts.chars));
    }
    if show_bytes {
        outputs.push(format!("{:>7}", counts.bytes));
    }
    if label.is_empty() {
        println!("{}", outputs.join(" "));
    } else {
        println!("{} {}", outputs.join(" "), label);
    }
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("wc", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "--version" {
            println!("wc (JeffUtils) 1.0");
            return;
        }
    }

    let mut show_lines = false;
    let mut show_words = false;
    let mut show_bytes = false;
    let mut show_chars = false;
    let mut files = Vec::new();

    for arg in &args {
        if arg.starts_with('-') && arg.len() > 1 {
            for c in arg.chars().skip(1) {
                match c {
                    'l' => show_lines = true,
                    'w' => show_words = true,
                    'c' => show_bytes = true,
                    'm' => show_chars = true,
                    _ => {
                        eprintln!("wc: invalid option -- '{}'", c);
                        print_usage();
                        std::process::exit(1);
                    }
                }
            }
        } else {
            files.push(arg.clone());
        }
    }

    if !show_lines && !show_words && !show_bytes && !show_chars {
        show_lines = true;
        show_words = true;
        show_bytes = true;
    }

    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;
    let mut total_chars = 0;

    if files.is_empty() {
        match count_reader(io::stdin().lock()) {
            Ok(c) => print_counts(&c, show_lines, show_words, show_bytes, show_chars, ""),
            Err(e) => {
                eprintln!("wc: error reading stdin: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        for filename in &files {
            let res = if filename == "-" {
                count_reader(io::stdin().lock())
            } else {
                match File::open(filename) {
                    Ok(f) => count_reader(BufReader::new(f)),
                    Err(e) => {
                        eprintln!("wc: {}: {}", filename, e);
                        continue;
                    }
                }
            };

            match res {
                Ok(c) => {
                    total_lines += c.lines;
                    total_words += c.words;
                    total_bytes += c.bytes;
                    total_chars += c.chars;
                    print_counts(&c, show_lines, show_words, show_bytes, show_chars, filename);
                }
                Err(e) => {
                    eprintln!("wc: error reading '{}': {}", filename, e);
                }
            }
        }

        if files.len() > 1 {
            let total_counts = Counts {
                lines: total_lines,
                words: total_words,
                bytes: total_bytes,
                chars: total_chars,
            };
            print_counts(&total_counts, show_lines, show_words, show_bytes, show_chars, "total");
        }
    }
}

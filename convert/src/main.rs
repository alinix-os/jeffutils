use std::io::{self, Read, Write};
use std::process::exit;

const VERSION: &str = "0.1.0";

fn print_help() {
    eprintln!("convert - translate or delete characters");
    eprintln!("Usage: convert SET1 SET2    translate characters in SET1 to SET2");
    eprintln!("       convert -d SET      delete characters in SET");
    eprintln!("       convert -h          print this help");
    eprintln!("       convert -v          print version");
    eprintln!();
    eprintln!("SET1 and SET2 are character sequences:");
    eprintln!("  Individual characters: abc");
    eprintln!("  Ranges: a-z, A-Z, 0-9");
    eprintln!("  Octal: \\NNN (three digits)");
    eprintln!("  Backslash: \\\\");
}

fn expand_set(set: &str) -> Vec<char> {
    let chars: Vec<char> = set.chars().collect();
    let mut expanded = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' {
            i += 1;
            if i >= chars.len() {
                expanded.push('\\');
                continue;
            }
            match chars[i] {
                '\\' => {
                    expanded.push('\\');
                    i += 1;
                }
                'n' => {
                    expanded.push('\n');
                    i += 1;
                }
                't' => {
                    expanded.push('\t');
                    i += 1;
                }
                '0'..='7' => {
                    let mut octal = String::new();
                    octal.push(chars[i]);
                    i += 1;
                    for _ in 0..2 {
                        if i < chars.len() && matches!(chars[i], '0'..='7') {
                            octal.push(chars[i]);
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    let val = u8::from_str_radix(&octal, 8).unwrap_or(0);
                    expanded.push(val as char);
                }
                _ => {
                    expanded.push(chars[i]);
                    i += 1;
                }
            }
        } else if i + 1 < chars.len() && chars[i + 1] == '-' && i + 2 < chars.len() {
            let start = chars[i] as u32;
            let end = chars[i + 2] as u32;
            if start <= end {
                for c in start..=end {
                    expanded.push(char::from_u32(c).unwrap_or('\0'));
                }
            } else {
                for c in (end..=start).rev() {
                    expanded.push(char::from_u32(c).unwrap_or('\0'));
                }
            }
            i += 3;
        } else {
            expanded.push(chars[i]);
            i += 1;
        }
    }

    expanded
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("convert: missing operand");
        eprintln!("Try 'convert -h' for more information.");
        exit(1);
    }

    match args[0].as_str() {
        "-h" | "--help" => {
            print_help();
            exit(0);
        }
        "-v" | "--version" => {
            eprintln!("convert {}", VERSION);
            exit(0);
        }
        "-d" => {
            if args.len() < 2 {
                eprintln!("convert: option -d requires an argument");
                exit(1);
            }
            let set = expand_set(&args[1]);
            let delete_set: std::collections::HashSet<char> = set.into_iter().collect();

            let mut input = String::new();
            io::stdin().read_to_string(&mut input).unwrap_or_else(|e| {
                eprintln!("convert: read error: {}", e);
                exit(1);
            });

            let stdout = io::stdout();
            let mut out = stdout.lock();
            for ch in input.chars() {
                if !delete_set.contains(&ch) {
                    write!(out, "{}", ch).unwrap();
                }
            }
        }
        a if a.starts_with('-') && !a.starts_with("--") => {
            eprintln!("convert: unknown option '{}'", a);
            exit(1);
        }
        _ => {
            if args.len() < 2 {
                eprintln!("convert: missing SET2 operand");
                exit(1);
            }
            let set1 = expand_set(&args[0]);
            let set2 = expand_set(&args[1]);

            let _max_len = set1.len().max(set2.len());
            let mut map: std::collections::HashMap<char, char> = std::collections::HashMap::new();
            for (i, &ch) in set1.iter().enumerate() {
                if i < set2.len() {
                    map.insert(ch, set2[i]);
                }
            }

            let mut input = String::new();
            io::stdin().read_to_string(&mut input).unwrap_or_else(|e| {
                eprintln!("convert: read error: {}", e);
                exit(1);
            });

            let stdout = io::stdout();
            let mut out = stdout.lock();
            for ch in input.chars() {
                if let Some(&mapped) = map.get(&ch) {
                    write!(out, "{}", mapped).unwrap();
                } else {
                    write!(out, "{}", ch).unwrap();
                }
            }
        }
    }
}

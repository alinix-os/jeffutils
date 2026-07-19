use std::env;

fn print_usage() {
    eprintln!("Usage: format FORMAT [ARG]...");
    eprintln!("Format and print arguments according to FORMAT.");
    eprintln!();
    eprintln!("Supports: %s %d %i %o %x %X %f %c %%%%");
    eprintln!("Escape sequences: \\n \\t \\\\ \\0NNN");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help       display this help and exit");
    eprintln!("  -v, --version    display version and exit");
}

fn unescape(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n' => { result.push('\n'); i += 2; }
                't' => { result.push('\t'); i += 2; }
                '\\' => { result.push('\\'); i += 2; }
                '0' => {
                    // Parse octal \0NNN
                    let mut octal = String::new();
                    let mut j = i + 2;
                    while j < chars.len() && chars[j].is_ascii_digit() && chars[j] <= '7' && j < i + 5 {
                        octal.push(chars[j]);
                        j += 1;
                    }
                    if let Ok(code) = u32::from_str_radix(&octal, 8) {
                        if let Some(c) = char::from_u32(code) {
                            result.push(c);
                        }
                    }
                    i = j;
                }
                _ => { result.push(chars[i]); i += 1; }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

fn format_string(fmt: &str, args: &[String]) -> String {
    let mut result = String::new();
    let fmt_chars: Vec<char> = fmt.chars().collect();
    let mut arg_idx = 0;
    let mut i = 0;

    while i < fmt_chars.len() {
        if fmt_chars[i] == '%' {
            if i + 1 < fmt_chars.len() {
                match fmt_chars[i + 1] {
                    '%' => { result.push('%'); i += 2; }
                    's' => {
                        if arg_idx < args.len() {
                            result.push_str(&args[arg_idx]);
                            arg_idx += 1;
                        }
                        i += 2;
                    }
                    'd' | 'i' => {
                        if arg_idx < args.len() {
                            match args[arg_idx].parse::<i64>() {
                                Ok(n) => result.push_str(&n.to_string()),
                                Err(_) => result.push_str(&args[arg_idx]),
                            }
                            arg_idx += 1;
                        }
                        i += 2;
                    }
                    'o' => {
                        if arg_idx < args.len() {
                            match args[arg_idx].parse::<u64>() {
                                Ok(n) => result.push_str(&format!("{:o}", n)),
                                Err(_) => result.push_str(&args[arg_idx]),
                            }
                            arg_idx += 1;
                        }
                        i += 2;
                    }
                    'x' => {
                        if arg_idx < args.len() {
                            match args[arg_idx].parse::<u64>() {
                                Ok(n) => result.push_str(&format!("{:x}", n)),
                                Err(_) => result.push_str(&args[arg_idx]),
                            }
                            arg_idx += 1;
                        }
                        i += 2;
                    }
                    'X' => {
                        if arg_idx < args.len() {
                            match args[arg_idx].parse::<u64>() {
                                Ok(n) => result.push_str(&format!("{:X}", n)),
                                Err(_) => result.push_str(&args[arg_idx]),
                            }
                            arg_idx += 1;
                        }
                        i += 2;
                    }
                    'f' => {
                        if arg_idx < args.len() {
                            match args[arg_idx].parse::<f64>() {
                                Ok(n) => result.push_str(&format!("{}", n)),
                                Err(_) => result.push_str(&args[arg_idx]),
                            }
                            arg_idx += 1;
                        }
                        i += 2;
                    }
                    'c' => {
                        if arg_idx < args.len() {
                            if let Some(ch) = args[arg_idx].chars().next() {
                                result.push(ch);
                            }
                            arg_idx += 1;
                        }
                        i += 2;
                    }
                    _ => {
                        result.push(fmt_chars[i]);
                        i += 1;
                    }
                }
            } else {
                result.push(fmt_chars[i]);
                i += 1;
            }
        } else {
            result.push(fmt_chars[i]);
            i += 1;
        }
    }

    result
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("format (JeffUtils) 1.0");
        return;
    }

    if args.is_empty() {
        eprintln!("format: missing FORMAT operand");
        eprintln!("Try 'format --help' for more information.");
        std::process::exit(1);
    }

    let fmt_raw = &args[0];
    let fmt = unescape(fmt_raw);
    let fmt_args = &args[1..];

    let output = format_string(&fmt, fmt_args);
    print!("{}", output);
}

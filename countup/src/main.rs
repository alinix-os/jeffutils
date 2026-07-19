use std::env;
use std::process;

fn usage() {
    eprintln!("Usage: countup [FIRST [INCR]] LAST [-w] [-s SEP] [-f FORMAT]");
    eprintln!("  Print a sequence of numbers from FIRST to LAST.");
    process::exit(1);
}

fn version() {
    println!("countup 0.1.0");
    process::exit(0);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        version();
    }

    let mut positional: Vec<i64> = Vec::new();
    let mut width_mode = false;
    let mut separator = "\n".to_string();
    let mut format_str: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-w" => width_mode = true,
            "-s" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("-s requires an argument");
                    process::exit(1);
                }
                separator = args[i].clone();
            }
            "-f" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("-f requires an argument");
                    process::exit(1);
                }
                format_str = Some(args[i].clone());
            }
            _ => {
                if let Ok(n) = args[i].parse::<i64>() {
                    positional.push(n);
                } else {
                    eprintln!("Invalid number: {}", args[i]);
                    process::exit(1);
                }
            }
        }
        i += 1;
    }

    if positional.is_empty() {
        eprintln!("At least one argument required");
        usage();
    }

    let (first, incr, last) = match positional.len() {
        1 => (0, 1, positional[0]),
        2 => (positional[0], 1, positional[1]),
        3 => (positional[0], positional[1], positional[2]),
        _ => {
            eprintln!("Too many arguments");
            process::exit(1);
        }
    };

    if incr == 0 {
        eprintln!("Increment cannot be zero");
        process::exit(1);
    }

    let mut numbers: Vec<i64> = Vec::new();
    if incr > 0 {
        let mut n = first;
        while n <= last {
            numbers.push(n);
            n += incr;
        }
    } else {
        let mut n = first;
        while n >= last {
            numbers.push(n);
            n += incr;
        }
    }

    let max_width = if width_mode {
        numbers.iter().map(|n| format!("{}", n).len()).max().unwrap_or(1)
    } else {
        0
    };

    for (idx, n) in numbers.iter().enumerate() {
        if let Some(ref fmt) = format_str {
            print!("{}", fmt_replacement(fmt, *n));
        } else if width_mode {
            print!("{:0>width$}", n, width = max_width);
        } else {
            print!("{}", n);
        }
        if idx < numbers.len() - 1 {
            print!("{}", separator);
        } else {
            println!();
        }
    }
}

fn fmt_replacement(fmt: &str, val: i64) -> String {
    let mut result = String::new();
    let chars: Vec<char> = fmt.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' && i + 1 < chars.len() {
            if chars[i + 1] == '%' {
                result.push('%');
                i += 2;
                continue;
            }
            // Parse format specifier
            let mut spec = String::from("%");
            i += 1;
            // flags
            while i < chars.len() && "-+ #0".contains(chars[i]) {
                spec.push(chars[i]);
                i += 1;
            }
            // width
            while i < chars.len() && chars[i].is_ascii_digit() {
                spec.push(chars[i]);
                i += 1;
            }
            // precision
            if i < chars.len() && chars[i] == '.' {
                spec.push(chars[i]);
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    spec.push(chars[i]);
                    i += 1;
                }
            }
            // length modifier
            if i < chars.len() && "hlLzjt".contains(chars[i]) {
                spec.push(chars[i]);
                i += 1;
            }
            // conversion
            if i < chars.len() {
                spec.push(chars[i]);
                i += 1;
            }
            // Apply conversion
            let last_char = spec.chars().last().unwrap_or('?');
            match last_char {
                'd' | 'i' | 'o' | 'u' | 'x' | 'X' => {
                    // integer formatting
                    let _fmt_clean = format!("{}{}", spec[..spec.len()-1].trim_start_matches('0'), last_char);
                    let _prefix_len = spec.len() - spec_clean_last(&spec);
                    let width_str: String = spec.chars().skip_while(|c| *c != '%' && *c != '0').take_while(|c| c.is_ascii_digit()).collect();
                    let width: usize = width_str.parse().unwrap_or(0);
                    let fill = if spec.contains('0') && !spec.contains('-') { '0' } else { ' ' };
                    let use_upper = last_char == 'X';
                    let _base: u64 = match last_char {
                        'o' => 8,
                        'x' | 'X' => 16,
                        _ => 10,
                    };
                    let signed = matches!(last_char, 'd' | 'i');
                    let mut num_str = if signed {
                        format!("{}", val)
                    } else {
                        format!("{}", val as u64)
                    };
                    if use_upper {
                        num_str = num_str.to_uppercase();
                    }
                    if width > num_str.len() {
                        let _pad: String = std::iter::repeat(fill).take(width - num_str.len()).collect();
                    }
                    if spec.contains('-') {
                        // left align
                        let pad_len = if width > num_str.len() { width - num_str.len() } else { 0 };
                        let pad: String = std::iter::repeat(' ').take(pad_len).collect();
                        result.push_str(&num_str);
                        result.push_str(&pad);
                    } else {
                        let pad_len = if width > num_str.len() { width - num_str.len() } else { 0 };
                        let pad: String = std::iter::repeat(fill).take(pad_len).collect();
                        result.push_str(&pad);
                        result.push_str(&num_str);
                    }
                }
                's' => {
                    let pad: usize = spec[1..spec.len()-1].parse().unwrap_or(0);
                    let s = val.to_string();
                    if spec.contains('-') {
                        result.push_str(&s);
                        for _ in 0..pad.saturating_sub(s.len()) {
                            result.push(' ');
                        }
                    } else {
                        for _ in 0..pad.saturating_sub(s.len()) {
                            result.push(' ');
                        }
                        result.push_str(&s);
                    }
                }
                'c' => {
                    let c = val as u8 as char;
                    result.push(c);
                }
                _ => {
                    result.push_str(&spec);
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

fn spec_clean_last(spec: &str) -> usize {
    spec.len()
}

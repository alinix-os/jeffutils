use std::env;
use std::io::{self, Write};

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("echo", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    let mut omit_newline = false;
    let mut enable_escapes = false;
    let mut start_idx = 0;

    while start_idx < args.len() {
        let arg = &args[start_idx];
        if arg.starts_with('-') && arg.len() > 1 {
            let mut valid_opt = true;
            let mut n = false;
            let mut e = false;
            let mut big_e = false;

            for c in arg.chars().skip(1) {
                match c {
                    'n' => n = true,
                    'e' => e = true,
                    'E' => big_e = true,
                    _ => {
                        valid_opt = false;
                        break;
                    }
                }
            }

            if valid_opt {
                if n { omit_newline = true; }
                if e { enable_escapes = true; }
                if big_e { enable_escapes = false; }
                start_idx += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let mut stdout = io::stdout().lock();
    for i in start_idx..args.len() {
        let mut word = args[i].clone();
        if enable_escapes {
            word = interpret_escapes(&word);
        }
        let _ = stdout.write_all(word.as_bytes());
        if i < args.len() - 1 {
            let _ = stdout.write_all(b" ");
        }
    }

    if !omit_newline {
        let _ = stdout.write_all(b"\n");
    }
    let _ = stdout.flush();
}

fn interpret_escapes(s: &str) -> String {
    let mut res = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('\\') => { res.push('\\'); chars.next(); }
                Some('a') => { res.push('\x07'); chars.next(); }
                Some('b') => { res.push('\x08'); chars.next(); }
                Some('e') => { res.push('\x1B'); chars.next(); }
                Some('f') => { res.push('\x0C'); chars.next(); }
                Some('n') => { res.push('\n'); chars.next(); }
                Some('r') => { res.push('\r'); chars.next(); }
                Some('t') => { res.push('\t'); chars.next(); }
                Some('v') => { res.push('\x0B'); chars.next(); }
                Some('0') => {
                    chars.next();
                    let mut octal = String::new();
                    for _ in 0..3 {
                        if let Some(&oc) = chars.peek() {
                            if oc.is_ascii_digit() && oc < '8' {
                                octal.push(oc);
                                chars.next();
                            } else { break; }
                        }
                    }
                    if let Ok(byte) = u8::from_str_radix(&octal, 8) {
                        res.push(byte as char);
                    }
                }
                Some('x') => {
                    chars.next();
                    let mut hex = String::new();
                    for _ in 0..2 {
                        if let Some(&hc) = chars.peek() {
                            if hc.is_ascii_hexdigit() {
                                hex.push(hc);
                                chars.next();
                            } else { break; }
                        }
                    }
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        res.push(byte as char);
                    }
                }
                _ => res.push('\\'),
            }
        } else {
            res.push(c);
        }
    }
    res
}

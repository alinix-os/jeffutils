use std::env;
use std::io::{self, BufRead, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    eprintln!("Usage: unitformat [OPTIONS] [NUMBER...]");
    eprintln!("Format numbers with SI/IEC units.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --to=UNIT      Output unit: auto,iec,iec-i,si,none (default: auto)");
    eprintln!("  --from=UNIT    Input unit: auto,iec,iec-i,si,none (default: none)");
    eprintln!("  -d DIGITS      Decimal digits (default: auto)");
    eprintln!("  -h, --help     Show this help message");
    eprintln!("  -v, --version  Show version");
}

#[derive(Clone, Copy, PartialEq)]
enum UnitMode {
    Auto,
    Iec,
    IecI,
    Si,
    None,
}

fn parse_unit(s: &str) -> Option<UnitMode> {
    match s.to_lowercase().as_str() {
        "auto" => Some(UnitMode::Auto),
        "iec" => Some(UnitMode::Iec),
        "iec-i" => Some(UnitMode::IecI),
        "si" => Some(UnitMode::Si),
        "none" => Some(UnitMode::None),
        _ => None,
    }
}

fn parse_number(s: &str) -> Option<f64> {
    let s = s.trim();
    s.parse::<f64>().ok()
}

fn format_auto(value: f64, digits: Option<usize>) -> String {
    if value.abs() >= 10f64.powi(60) {
        let exp = ((value.abs().log2()) / 10.0 * 3.0) as u32;
        let base = value / 10f64.powi((exp / 3 * 10 / 3) as i32);
        return format!("{:.prec$}e{}", base, exp, prec = digits.unwrap_or(1));
    }

    if value.abs() < 1000.0 {
        return match digits {
            Some(d) => format!("{:.prec$}", value, prec = d),
            None => {
                if value == value.floor() && value.abs() < 1e15 {
                    format!("{}", value as i64)
                } else {
                    format!("{}", value)
                }
            }
        };
    }

    let si_prefixes = [
        (1e18, "E"),
        (1e15, "P"),
        (1e12, "T"),
        (1e9, "G"),
        (1e6, "M"),
        (1e3, "k"),
    ];

    for &(threshold, prefix) in &si_prefixes {
        if value.abs() >= threshold {
            let scaled = value / threshold;
            return match digits {
                Some(d) => format!("{:.prec$}{}", scaled, prefix, prec = d),
                None => format!("{}{}", scaled, prefix),
            };
        }
    }

    match digits {
        Some(d) => format!("{:.prec$}", value, prec = d),
        None => format!("{}", value),
    }
}

fn format_iec(value: f64, digits: Option<usize>) -> String {
    let iec_prefixes = [
        (1024f64.powi(6), "Ei"),
        (1024f64.powi(5), "Pi"),
        (1024f64.powi(4), "Ti"),
        (1024f64.powi(3), "Gi"),
        (1024f64.powi(2), "Mi"),
        (1024f64, "Ki"),
    ];

    if value.abs() < 1024.0 {
        return match digits {
            Some(d) => format!("{:.prec$}", value, prec = d),
            None => {
                if value == value.floor() && value.abs() < 1e15 {
                    format!("{}", value as i64)
                } else {
                    format!("{}", value)
                }
            }
        };
    }

    for &(threshold, prefix) in &iec_prefixes {
        if value.abs() >= threshold {
            let scaled = value / threshold;
            return match digits {
                Some(d) => format!("{:.prec$}{}", scaled, prefix, prec = d),
                None => format!("{}{}", scaled, prefix),
            };
        }
    }

    match digits {
        Some(d) => format!("{:.prec$}", value, prec = d),
        None => format!("{}", value),
    }
}

fn format_iec_i(value: f64, digits: Option<usize>) -> String {
    let iec_prefixes = [
        (1024f64.powi(6), "Ei"),
        (1024f64.powi(5), "Pi"),
        (1024f64.powi(4), "Ti"),
        (1024f64.powi(3), "Gi"),
        (1024f64.powi(2), "Mi"),
        (1024f64, "Ki"),
    ];

    if value.abs() < 1024.0 {
        return match digits {
            Some(d) => format!("{:.prec$}", value, prec = d),
            None => {
                if value == value.floor() && value.abs() < 1e15 {
                    format!("{}", value as i64)
                } else {
                    format!("{}", value)
                }
            }
        };
    }

    for &(threshold, prefix) in &iec_prefixes {
        if value.abs() >= threshold {
            let scaled = value / threshold;
            return match digits {
                Some(d) => format!("{:.prec$}{}", scaled, prefix, prec = d),
                None => format!("{}{}", scaled, prefix),
            };
        }
    }

    match digits {
        Some(d) => format!("{:.prec$}", value, prec = d),
        None => format!("{}", value),
    }
}

fn format_si(value: f64, digits: Option<usize>) -> String {
    let si_prefixes = [
        (1e18, "E"),
        (1e15, "P"),
        (1e12, "T"),
        (1e9, "G"),
        (1e6, "M"),
        (1e3, "k"),
    ];

    if value.abs() < 1000.0 {
        return match digits {
            Some(d) => format!("{:.prec$}", value, prec = d),
            None => {
                if value == value.floor() && value.abs() < 1e15 {
                    format!("{}", value as i64)
                } else {
                    format!("{}", value)
                }
            }
        };
    }

    for &(threshold, prefix) in &si_prefixes {
        if value.abs() >= threshold {
            let scaled = value / threshold;
            return match digits {
                Some(d) => format!("{:.prec$}{}", scaled, prefix, prec = d),
                None => format!("{}{}", scaled, prefix),
            };
        }
    }

    match digits {
        Some(d) => format!("{:.prec$}", value, prec = d),
        None => format!("{}", value),
    }
}

fn format_value(value: f64, mode: UnitMode, digits: Option<usize>) -> String {
    match mode {
        UnitMode::Auto => format_auto(value, digits),
        UnitMode::Iec => format_iec(value, digits),
        UnitMode::IecI => format_iec_i(value, digits),
        UnitMode::Si => format_si(value, digits),
        UnitMode::None => match digits {
            Some(d) => format!("{:.prec$}", value, prec = d),
            None => format!("{}", value),
        },
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut to_mode = UnitMode::Auto;
    let mut from_mode = UnitMode::None;
    let mut digits: Option<usize> = None;
    let mut numbers = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-v" | "--version" => {
                println!("unitformat {VERSION}");
                return;
            }
            arg if arg.starts_with("--to=") => {
                let val = &arg[5..];
                to_mode = match parse_unit(val) {
                    Some(m) => m,
                    None => {
                        eprintln!("unitformat: unknown unit '{val}'");
                        std::process::exit(1);
                    }
                };
            }
            arg if arg.starts_with("--from=") => {
                let val = &arg[7..];
                from_mode = match parse_unit(val) {
                    Some(m) => m,
                    None => {
                        eprintln!("unitformat: unknown unit '{val}'");
                        std::process::exit(1);
                    }
                };
            }
            "-d" => {
                i += 1;
                if i < args.len() {
                    digits = Some(args[i].parse().unwrap_or_else(|_| {
                        eprintln!("unitformat: invalid digit count '{}'", args[i]);
                        std::process::exit(1);
                    }));
                } else {
                    eprintln!("unitformat: -d requires an argument");
                    std::process::exit(1);
                }
            }
            "--" => {
                i += 1;
                while i < args.len() {
                    numbers.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            _ if args[i].starts_with('-') => {
                eprintln!("unitformat: unknown option '{}'", args[i]);
                std::process::exit(1);
            }
            _ => numbers.push(args[i].clone()),
        }
        i += 1;
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut process = |num_str: &str, to_mode: UnitMode, from_mode: UnitMode, digits: Option<usize>| {
        let value = match parse_number(num_str) {
            Some(v) => v,
            None => {
                eprintln!("unitformat: invalid number '{num_str}'");
                return;
            }
        };
        let base_value = match from_mode {
            UnitMode::Iec => value * 1024.0f64.powi(3),
            UnitMode::IecI => value * 1024.0f64.powi(3),
            UnitMode::Si => value * 1000.0f64.powi(3),
            _ => value,
        };
        let result = format_value(base_value, to_mode, digits);
        writeln!(out, "{result}").ok();
    };

    if numbers.is_empty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            process(trimmed, to_mode, from_mode, digits);
        }
    } else {
        for num in &numbers {
            process(num, to_mode, from_mode, digits);
        }
    }
}

use std::time::Duration;

fn print_usage() {
    eprintln!("Usage: sleep <duration>[s|m|h|d]...  or run sleep without arguments to suspend system.");
    eprintln!("Pause for NUMBER of seconds. Suffixes s (seconds), m (minutes), h (hours), d (days) are supported.");
}

fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let (num_str, unit) = if s.ends_with(|c: char| c.is_alphabetic()) {
        let (num, unit) = s.split_at(s.len() - 1);
        (num, unit)
    } else {
        (s, "s")
    };

    let value: f64 = num_str.parse().ok()?;
    if value < 0.0 {
        return None;
    }
    match unit {
        "s" => Some(Duration::from_secs_f64(value)),
        "m" => Some(Duration::from_secs_f64(value * 60.0)),
        "h" => Some(Duration::from_secs_f64(value * 3600.0)),
        "d" => Some(Duration::from_secs_f64(value * 86400.0)),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("  --help, -h  Show this help message");
                println!("  --version   Show version information");
                return;
            }
            "--version" => {
                println!("sleep version 0.2.0");
                return;
            }
            _ => {}
        }
    }

    if args.is_empty() {
        // Suspend the system (as per JeffUtils custom specification)
        #[cfg(target_os = "linux")]
        let status = std::process::Command::new("systemctl").args(["suspend"]).status();

        #[cfg(target_os = "macos")]
        let status = std::process::Command::new("pmset").args(["sleepnow"]).status();

        #[cfg(target_os = "windows")]
        let status = std::process::Command::new("rundll32.exe").args(["powrprof.dll,SetSuspendState", "0,1,0"]).status();

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let status = Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Sleep not supported on this platform"));

        match status {
            Ok(s) if s.success() => {},
            Ok(_) => {
                eprintln!("Error: sleep command failed (permission denied or not available)");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error: could not execute sleep: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        let mut total_duration = Duration::ZERO;
        for arg in &args {
            if let Some(d) = parse_duration(arg) {
                total_duration += d;
            } else {
                eprintln!("Error: invalid duration '{}'. Use format like 30s, 0.5s, 5m, 1h", arg);
                std::process::exit(1);
            }
        }
        std::thread::sleep(total_duration);
    }
}

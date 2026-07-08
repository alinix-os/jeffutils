fn print_usage() {
    eprintln!("Usage: {} [s|h|m]", std::env::args().nth(0).unwrap_or_else(|| "sleep".into()));
}

fn parse_duration(s: &str) -> Option<u64> {
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

    let value: u64 = num_str.parse().ok()?;
    match unit {
        "s" => Some(value),
        "m" => Some(value * 60),
        "h" => Some(value * 3600),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Puts the system to sleep.");
                println!("  <duration>  Sleep duration (e.g., 30s, 5m, 1h)");
                println!("  --help, -h  Show this help message");
                println!("  --version   Show version information");
                return;
            }
            "--version" => {
                println!("sleep version 0.1.0");
                return;
            }
            _ => {}
        }
    }

    if args.is_empty() {
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
        let duration = parse_duration(&args[0]).unwrap_or_else(|| {
            eprintln!("Error: invalid duration '{}'. Use format like 30s, 5m, 1h", args[0]);
            std::process::exit(1);
        });
        std::thread::sleep(std::time::Duration::from_secs(duration));
    }
}

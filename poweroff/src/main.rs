fn print_usage() {
    eprintln!("Usage: {}", std::env::args().nth(0).unwrap_or_else(|| "poweroff".into()));
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Powers off the system.");
                println!("  --help, -h  Show this help message");
                println!("  --version   Show version information");
                return;
            }
            "--version" => {
                println!("poweroff version 0.1.0");
                return;
            }
            _ => {
                print_usage();
                std::process::exit(1);
            }
        }
    }

    #[cfg(target_os = "linux")]
    let status = std::process::Command::new("systemctl").args(["poweroff"]).status();

    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("shutdown").args(["-h", "now"]).status();

    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("shutdown").args(["/s", "/t", "0"]).status();

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let status = std::process::Command::new("poweroff").status();

    match status {
        Ok(s) if s.success() => {},
        Ok(_) => {
            eprintln!("Error: poweroff command failed (permission denied or not available)");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: could not execute poweroff: {}", e);
            std::process::exit(1);
        }
    }
}

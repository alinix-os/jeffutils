fn print_usage() {
    eprintln!("Usage: {}", std::env::args().nth(0).unwrap_or_else(|| "reboot".into()));
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("reboot", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Reboots the system.");
                println!("  --help, -h  Show this help message");
                println!("  --version   Show version information");
                return;
            }
            "--version" => {
                println!("reboot version 0.1.0");
                return;
            }
            _ => {
                print_usage();
                std::process::exit(1);
            }
        }
    }

    #[cfg(target_os = "linux")]
    let status = std::process::Command::new("systemctl").args(["reboot"]).status();

    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("shutdown").args(["-r", "now"]).status();

    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("shutdown").args(["/r", "/t", "0"]).status();

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let status = std::process::Command::new("reboot").status();

    match status {
        Ok(s) if s.success() => {},
        Ok(_) => {
            eprintln!("Error: reboot command failed (permission denied or not available)");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: could not execute reboot: {}", e);
            std::process::exit(1);
        }
    }
}

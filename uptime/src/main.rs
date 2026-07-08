fn print_usage() {
    eprintln!("Usage: {}", std::env::args().nth(0).unwrap_or_else(|| "uptime".into()));
}

#[cfg(target_os = "linux")]
fn get_uptime_seconds() -> Result<u64, String> {
    let content = std::fs::read_to_string("/proc/uptime").map_err(|e| e.to_string())?;
    let secs = content.split_whitespace().next().unwrap_or("0").parse::<f64>().map_err(|e| e.to_string())?;
    Ok(secs as u64)
}

#[cfg(not(target_os = "linux"))]
fn get_uptime_seconds() -> Result<u64, String> {
    let output = std::process::Command::new("sysctl").arg("-n").arg("kern.boottime").output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let boot_secs = stdout.split(|c: char| !c.is_ascii_digit()).filter_map(|s| s.parse::<i64>().ok()).next().ok_or("Could not parse boot time")?;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    Ok((now - boot_secs) as u64)
}

fn format_duration(total_secs: u64) -> String {
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    let mut parts = Vec::new();
    if days > 0 { parts.push(format!("{}d", days)); }
    if hours > 0 { parts.push(format!("{}h", hours)); }
    if minutes > 0 { parts.push(format!("{}m", minutes)); }
    parts.push(format!("{}s", seconds));
    parts.join(" ")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        if arg == "--help" || arg == "-h" {
            print_usage();
            println!("Shows system uptime.");
            println!("  --help, -h  Show this help message");
            println!("  --version   Show version information");
            return;
        }
        if arg == "--version" {
            println!("uptime version 0.1.0");
            return;
        }
    }

    if !args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    match get_uptime_seconds() {
        Ok(secs) => println!("{}", format_duration(secs)),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

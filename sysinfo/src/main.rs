fn print_usage() {
    eprintln!("Usage: {}", std::env::args().nth(0).unwrap_or_else(|| "sysinfo".into()));
}

fn get_os_name() -> String {
    std::env::consts::OS.to_string()
}

fn get_hostname() -> String {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/sys/kernel/hostname")
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".into())
    }
    #[cfg(not(target_os = "linux"))]
    {
        std::process::Command::new("hostname")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".into())
    }
}

fn get_kernel() -> String {
    #[cfg(target_os = "linux")]
    {
        let ostype = std::fs::read_to_string("/proc/sys/kernel/ostype")
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Linux".into());
        let release = std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".into());
        format!("{} {}", ostype, release)
    }
    #[cfg(target_os = "macos")]
    {
        let release = std::process::Command::new("uname").arg("-r").output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".into());
        format!("Darwin {}", release)
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("OS").unwrap_or_else(|_| "Windows".into())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "unknown".into()
    }
}

#[cfg(target_os = "linux")]
fn get_cpu_info() -> String {
    let content = match std::fs::read_to_string("/proc/cpuinfo") {
        Ok(s) => s,
        Err(_) => return "unknown".into(),
    };
    content
        .lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

#[cfg(not(target_os = "linux"))]
fn get_cpu_info() -> String {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".into())
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| "unknown".into())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        "unknown".into()
    }
}

#[cfg(target_os = "linux")]
fn get_memory() -> String {
    let content = std::fs::read_to_string("/proc/meminfo").ok().unwrap_or_default();
    let total = content.lines()
        .find(|l| l.starts_with("MemTotal:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    let free = content.lines()
        .find(|l| l.starts_with("MemAvailable:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    let total_mb = total / 1024;
    let free_mb = free / 1024;
    format!("{} MB total, {} MB available", total_mb, free_mb)
}

#[cfg(not(target_os = "linux"))]
fn get_memory() -> String {
    #[cfg(target_os = "macos")]
    {
        let total_bytes = std::process::Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let total_mb = total_bytes / 1024 / 1024;
        format!("{} MB total", total_mb)
    }
    #[cfg(not(target_os = "macos"))]
    {
        "unknown".into()
    }
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("sysinfo", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Displays system information.");
                println!("  --help, -h  Show this help message");
                println!("  --version   Show version information");
                return;
            }
            "--version" => {
                println!("sysinfo version 0.1.0");
                return;
            }
            _ => {
                print_usage();
                std::process::exit(1);
            }
        }
    }

    println!("OS       : {}", get_os_name());
    println!("Hostname : {}", get_hostname());
    println!("Kernel   : {}", get_kernel());
    println!("CPU      : {}", get_cpu_info());
    println!("Memory   : {}", get_memory());
}

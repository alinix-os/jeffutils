use std::path::PathBuf;

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config").join("jeffutils").join("clock.conf")
}

fn load_format() -> Option<String> {
    let path = config_path();
    if path.exists() {
        std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
    } else {
        None
    }
}

fn save_format(format: &str) {
    if let Some(parent) = config_path().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(config_path(), format);
}

fn format_now(format: &str) -> String {
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let mut now: libc::time_t = 0;
    unsafe { libc::time(&mut now) };
    unsafe { libc::localtime_r(&now, &mut tm) };

    let months = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];

    format
        .replace("HH", &format!("{:02}", tm.tm_hour))
        .replace("mm", &format!("{:02}", tm.tm_min))
        .replace("ss", &format!("{:02}", tm.tm_sec))
        .replace("dd", &format!("{:02}", tm.tm_mday))
        .replace("MM", &format!("{:02}", tm.tm_mon + 1))
        .replace("yyyy", &format!("{:04}", tm.tm_year + 1900))
        .replace("Month", months[tm.tm_mon as usize])
        .replace("DD", &format!("{:03}", tm.tm_yday + 1))
}

fn print_usage() {
    eprintln!("Usage: {} [formato] [--config <formato>]", std::env::args().nth(0).unwrap_or_else(|| "clock".into()));
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("clock", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        if arg == "--help" || arg == "-h" {
            print_usage();
            println!("Displays the current date and time.");
            println!("  --config <format>  Set default format");
            println!("  --version         Show version information");
            println!("");
            println!("Format specifiers:");
            println!("  HH        Hours (00-23)");
            println!("  mm        Minutes (00-59)");
            println!("  ss        Seconds (00-59)");
            println!("  dd        Day (01-31)");
            println!("  MM        Month (01-12)");
            println!("  yyyy      Year (4 digits)");
            println!("  Month     Full month name");
            println!("  DD        Day of year (001-366)");
            println!("  --help, -h Show this help message");
            return;
        }
        if arg == "--version" {
            println!("clock version 0.1.0");
            return;
        }
    }

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--config" {
            if i + 1 < args.len() {
                save_format(&args[i + 1]);
                println!("Default format set to: {}", args[i + 1]);
                return;
            } else {
                eprintln!("Error: --config requires a format argument");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let format = if !args.is_empty() && !args[0].starts_with("--") {
        args[0].clone()
    } else {
        load_format().unwrap_or_else(|| "dd/MM/yyyy HH:mm:ss".to_string())
    };

    println!("{}", format_now(&format));
}

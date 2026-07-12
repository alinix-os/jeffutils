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
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let (y, m, d) = days_to_date(days);
    let months = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];

    format
        .replace("HH", &format!("{:02}", hours))
        .replace("mm", &format!("{:02}", minutes))
        .replace("ss", &format!("{:02}", seconds))
        .replace("dd", &format!("{:02}", d))
        .replace("MM", &format!("{:02}", m))
        .replace("yyyy", &format!("{:04}", y))
        .replace("Month", months[m as usize - 1])
        .replace("DD", &format!("{:03}", days_since_year_start(y, m, d)))
}

fn days_to_date(mut days: u64) -> (u64, u64, u64) {
    let mut year: u64 = 1970;
    loop {
        let ydays = if is_leap(year) { 366 } else { 365 };
        if days < ydays {
            break;
        }
        days -= ydays;
        year += 1;
    }
    let leap = is_leap(year);
    let mdays = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month: u64 = 1;
    for &md in &mdays {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_since_year_start(year: u64, month: u64, day: u64) -> u64 {
    let mdays = [31, if is_leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    mdays[..(month as usize - 1)].iter().sum::<u64>() + day
}

fn print_usage() {
    eprintln!("Usage: {} [formato] [--config <formato>]", std::env::args().nth(0).unwrap_or_else(|| "clock".into()));
}

fn main() {
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

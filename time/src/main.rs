use std::env;
use std::process::Command;
use std::time::Instant;

fn main() {
    if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") {
        jutils_core::print_version("time", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }
    let args: Vec<String> = env::args().skip(1).collect();
    for arg in &args {
        if arg == "-h" || arg == "--help" {
            println!("Usage: time <command> [args...]");
            println!("Measure the execution time of a command.");
            return;
        }
        if arg == "--version" {
            println!("time (JeffUtils) 1.0");
            return;
        }
    }
    if args.is_empty() {
        println!("Uso: time <comando> [args...]");
        return;
    }
    let cmd = &args[0];
    let cmd_args = &args[1..];

    let mut usage_before = unsafe { std::mem::zeroed::<libc::rusage>() };
    unsafe {
        libc::getrusage(libc::RUSAGE_CHILDREN, &mut usage_before);
    }
    let start = Instant::now();

    let status = Command::new(cmd)
        .args(cmd_args)
        .status();

    let duration = start.elapsed().as_secs_f64();
    let mut usage_after = unsafe { std::mem::zeroed::<libc::rusage>() };
    unsafe {
        libc::getrusage(libc::RUSAGE_CHILDREN, &mut usage_after);
    }

    match status {
        Ok(s) => {
            let user_time = (usage_after.ru_utime.tv_sec - usage_before.ru_utime.tv_sec) as f64
                + (usage_after.ru_utime.tv_usec - usage_before.ru_utime.tv_usec) as f64 / 1_000_000.0;
            let sys_time = (usage_after.ru_stime.tv_sec - usage_before.ru_stime.tv_sec) as f64
                + (usage_after.ru_stime.tv_usec - usage_before.ru_stime.tv_usec) as f64 / 1_000_000.0;

            eprintln!("real    {}", format_time(duration));
            eprintln!("user    {}", format_time(user_time));
            eprintln!("sys     {}", format_time(sys_time));

            if !s.success() {
                std::process::exit(s.code().unwrap_or(1));
            }
        }
        Err(e) => {
            eprintln!("Erro ao executar comando: {}", e);
            std::process::exit(1);
        }
    }
}

fn get_decimal_separator() -> &'static str {
    for var in &["LC_NUMERIC", "LC_ALL", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            let val_lower = val.to_lowercase();
            if val_lower.starts_with("pt")
                || val_lower.starts_with("fr")
                || val_lower.starts_with("de")
                || val_lower.starts_with("es")
                || val_lower.starts_with("it")
                || val_lower.starts_with("ru")
                || val_lower.starts_with("nl")
                || val_lower.starts_with("da")
                || val_lower.starts_with("sv")
                || val_lower.starts_with("nb")
                || val_lower.starts_with("nn")
                || val_lower.starts_with("fi")
                || val_lower.starts_with("pl")
                || val_lower.starts_with("cs")
                || val_lower.starts_with("sk")
                || val_lower.starts_with("hu")
                || val_lower.starts_with("tr")
                || val_lower.starts_with("el")
            {
                return ",";
            }
        }
    }
    "."
}

fn format_time(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor() as u64;
    let remaining_seconds = seconds - (minutes as f64 * 60.0);
    let sec_str = format!("{:.3}", remaining_seconds);
    let sep = get_decimal_separator();
    let formatted_secs = if sep == "," {
        sec_str.replace('.', ",")
    } else {
        sec_str
    };
    format!("{}m{}s", minutes, formatted_secs)
}
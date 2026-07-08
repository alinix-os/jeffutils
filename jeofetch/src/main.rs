use std::env;
use crossterm::style::Stylize;
use sysinfo::System;

/// ASCII art logo "JA" displayed by jeofetch
const JA_LOGO: &[&str] = &[
    r"     ██╗ █████╗ ",
    r"     ██║██╔══██╗",
    r"     ██║███████║",
    r"██   ██║██╔══██║",
    r"╚█████╔╝██║  ██║",
    r" ╚════╝ ╚═╝  ╚═╝",
];

fn main() {
    run_jeofetch();
}

fn run_jeofetch() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os_name = System::name().unwrap_or_else(|| "Unknown OS".to_string());
    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let uptime_sec = System::uptime();
    let up_hours = uptime_sec / 3600;
    let up_mins = (uptime_sec % 3600) / 60;
    let hostname = System::host_name().unwrap_or_else(|| "localhost".to_string());
    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    let shell = "jsh";

    let cpu = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let total_mem = sys.total_memory() / 1024 / 1024;
    let used_mem = sys.used_memory() / 1024 / 1024;

    // Info lines paired with logo lines
    let info_lines: Vec<String> = vec![
        format!(
            "{}@{}",
            user.bold().green(),
            hostname.bold().green()
        ),
        format!("{:<10} {}", "OS:".cyan(), os_name),
        format!("{:<10} {}", "Kernel:".cyan(), kernel),
        format!("{:<10} {}h {}m", "Uptime:".cyan(), up_hours, up_mins),
        format!("{:<10} {}", "Shell:".cyan(), shell),
        format!("{:<10} {}", "CPU:".cyan(), cpu),
        format!(
            "{:<10} {} / {} MB",
            "Memory:".cyan(),
            used_mem,
            total_mem
        ),
    ];

    let logo_width = JA_LOGO.iter().map(|l| l.len()).max().unwrap_or(0);

    // Print each logo line alongside the info
    for (i, logo_line) in JA_LOGO.iter().enumerate() {
        let colored_logo = logo_line.bold().magenta().to_string();
        if let Some(info) = info_lines.get(i) {
            println!("{}  {}", colored_logo, info);
        } else {
            println!("{}", colored_logo);
        }
    }

    // Print any remaining info lines that exceed logo height
    for info in info_lines.iter().skip(JA_LOGO.len()) {
        println!("{:width$}  {}", "", info, width = logo_width);
    }

    println!();
}

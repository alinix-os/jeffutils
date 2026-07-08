#[derive(Clone, Copy, PartialEq)]
enum UnitSystem {
    Binary,
    Decimal,
}

fn print_usage() {
    eprintln!(
        "Usage: {} [--meminfo] [-h | -g | --giga] [-gib | --gib]",
        std::env::args().nth(0).unwrap_or_else(|| "memory".into())
    );
}

#[cfg(target_os = "linux")]
fn read_meminfo() -> String {
    std::fs::read_to_string("/proc/meminfo").unwrap_or_else(|_| "unavailable".into())
}

#[cfg(target_os = "linux")]
fn get_mem_value(key: &str) -> u64 {
    let content = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    content.lines()
        .find(|l| l.starts_with(key))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn format_size(kb: u64, system: UnitSystem) -> String {
    match system {
        UnitSystem::Binary => {
            if kb >= 1_048_576 {
                format!("{:.2} GiB", kb as f64 / 1_048_576.0)
            } else if kb >= 1024 {
                format!("{:.2} MiB", kb as f64 / 1024.0)
            } else {
                format!("{} KiB", kb)
            }
        }
        UnitSystem::Decimal => {
            let bytes = kb as f64 * 1024.0;
            if bytes >= 1_000_000_000.0 {
                format!("{:.2} GB", bytes / 1_000_000_000.0)
            } else if bytes >= 1_000_000.0 {
                format!("{:.2} MB", bytes / 1_000_000.0)
            } else if bytes >= 1000.0 {
                format!("{:.2} KB", bytes / 1000.0)
            } else {
                format!("{} B", bytes)
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut system = UnitSystem::Decimal;
    let mut show_meminfo = false;

    for arg in &args {
        match arg.as_str() {
            "--help" => {
                print_usage();
                println!("Shows memory usage information.");
                println!("  --meminfo          Show detailed /proc/meminfo (Linux)");
                println!("  -h, -g, --giga     Show in decimal gigabytes (GB/MB/KB) [default]");
                println!("  -gib, --gib        Show in binary gibibytes (GiB/MiB/KiB)");
                println!("  --help             Show this help message");
                println!("  --version          Show version information");
                return;
            }
            "--version" => {
                println!("memory version 0.1.0");
                return;
            }
            "--meminfo" => {
                show_meminfo = true;
            }
            "-h" | "-g" | "--giga" => {
                system = UnitSystem::Decimal;
            }
            "-gib" | "--gib" => {
                system = UnitSystem::Binary;
            }
            _ => {
                print_usage();
                std::process::exit(1);
            }
        }
    }

    if show_meminfo {
        #[cfg(target_os = "linux")]
        {
            println!("{}", read_meminfo());
            return;
        }
        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("Error: --meminfo is only available on Linux");
            std::process::exit(1);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let total = get_mem_value("MemTotal:");
        let free = get_mem_value("MemFree:");
        let available = get_mem_value("MemAvailable:");
        let buffers = get_mem_value("Buffers:");
        let cached = get_mem_value("Cached:");
        let sreclaimable = get_mem_value("SReclaimable:");

        let total_used = total.saturating_sub(free);
        let cache_buffers = buffers + cached + sreclaimable;

        println!("Memory Usage:");
        println!("  Total     : {}", format_size(total, system));
        println!("  Used      : {} (cache/buff: {})", format_size(total_used, system), format_size(cache_buffers, system));
        println!("  Free      : {}", format_size(free, system));
        println!("  Available : {}", format_size(available, system));

        let swap_total = get_mem_value("SwapTotal:");
        let swap_free = get_mem_value("SwapFree:");
        if swap_total > 0 {
            let swap_used = swap_total.saturating_sub(swap_free);
            println!("  Swap Total: {}", format_size(swap_total, system));
            println!("  Swap Used : {}", format_size(swap_used, system));
            println!("  Swap Free : {}", format_size(swap_free, system));
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let output = std::process::Command::new("vm_stat")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_else(|| "Memory info not available".into());
        println!("{}", output);
    }
}

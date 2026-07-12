use std::io::{self, Write};
use std::time::Duration;
use sysinfo::{System, CpuRefreshKind, ProcessRefreshKind};
use crossterm::{
    execute,
    terminal::{Clear, ClearType, size},
    style::{Color, Stylize, Print, ResetColor, SetForegroundColor},
    cursor::MoveTo,
};

fn main() {
    let mut sys = System::new_all();
    let mut stdout = io::stdout();
    let delay = Duration::from_secs(2);

    loop {
        // Refresh system metrics
        sys.refresh_all();
        
        let (width, height) = size().unwrap_or((80, 24));
        
        // Clear screen and move cursor to home position
        execute!(
            stdout,
            Clear(ClearType::All),
            MoveTo(0, 0)
        ).ok();

        // 1. Header (Retro-Futuristic Glassmorphism Vibe)
        println!("{}", " ── JUtils System Monitor (jtop) ──────────────────────────────── ".bold().cyan());
        
        // Hostname, Uptime & CPU load
        let uptime = System::uptime();
        let up_hours = uptime / 3600;
        let up_mins = (uptime % 3600) / 60;
        println!(
            "  Uptime: {}h {}m  |  CPUs: {}  |  Carga Global: {:.1}%",
            up_hours.to_string().bold().green(),
            up_mins.to_string().bold().green(),
            sys.cpus().len().to_string().bold().yellow(),
            sys.global_cpu_usage().to_string().bold().magenta()
        );

        // Memory bar
        let total_mem = sys.total_memory() / 1024 / 1024;
        let used_mem = sys.used_memory() / 1024 / 1024;
        let mem_pct = (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0;
        
        // Render stylized memory bar
        let bar_width = 30;
        let filled_chars = ((mem_pct / 100.0) * bar_width as f64).round() as usize;
        let empty_chars = bar_width - filled_chars;
        let bar = format!(
            "[{}{}]",
            "■".repeat(filled_chars).green(),
            " ".repeat(empty_chars)
        );
        println!(
            "  Memória: {} {} / {} MB ({:.1}%)",
            bar,
            used_mem.to_string().bold().green(),
            total_mem.to_string().bold().white(),
            mem_pct.to_string().bold().yellow()
        );

        println!("{}", " ───────────────────────────────────────────────────────────────── ".cyan());
        
        // 2. Process Table Header
        println!(
            "  {:>6}  {:<20}  {:>8}  {:>10}",
            "PID".bold().yellow(),
            "COMANDO".bold().white(),
            "CPU%".bold().yellow(),
            "MEMÓRIA".bold().yellow()
        );
        println!("{}", " ───────────────────────────────────────────────────────────────── ".cyan());

        // Sort and display top processes
        let mut processes: Vec<_> = sys.processes().values().collect();
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));

        let max_rows = if height > 8 { height - 8 } else { 10 };
        for proc in processes.iter().take(max_rows as usize) {
            let pid = proc.pid().as_u32();
            let name = proc.name().to_string_lossy();
            let cpu = proc.cpu_usage();
            let mem = proc.memory() / 1024; // KB to MB
            
            println!(
                "  {:>6}  {:<20}  {:>7.1}%  {:>8} MB",
                pid.to_string().green(),
                if name.len() > 20 { &name[..20] } else { &name },
                cpu,
                mem
            );
        }

        stdout.flush().ok();
        std::thread::sleep(delay);
    }
}

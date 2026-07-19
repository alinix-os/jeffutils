use std::time::Duration;
use sysinfo::System;

fn main() {
    let mut sys = System::new();
    sys.refresh_all();
    std::thread::sleep(Duration::from_millis(200));
    sys.refresh_all();

    let mut procs: Vec<_> = sys.processes().iter().collect();
    procs.sort_by_key(|(pid, _)| **pid);

    println!("{:>6} {:>8} {:<20}", "PID", "CPU%", "COMMAND");
    println!("{}", "-".repeat(40));
    for (pid, process) in procs {
        println!("{:>6} {:>7.1}% {:<20}", pid, process.cpu_usage(), process.name().to_string_lossy());
    }
}

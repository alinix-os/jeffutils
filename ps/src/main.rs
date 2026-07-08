use sysinfo::System;

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();
    println!("{:>6} {:>8} {:<20}", "PID", "CPU%", "COMANDO");
    println!("{}", "-".repeat(40));
    for (pid, process) in sys.processes() {
        println!("{:>6} {:>7.1}% {:<20}", pid, process.cpu_usage(), process.name().to_string_lossy());
    }
}
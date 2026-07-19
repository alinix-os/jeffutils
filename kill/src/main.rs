use std::path::Path;

fn print_usage() {
    eprintln!("Usage: {} <pid> [--signal <signame>]", std::env::args().nth(0).unwrap_or_else(|| "kill".into()));
}

#[cfg(unix)]
fn send_signal(pid: u32, signal: &str) -> Result<(), String> {
    let sig = match signal {
        "SIGTERM" | "TERM" | "15" => "15",
        "SIGKILL" | "KILL" | "9" => "9",
        "SIGHUP" | "HUP" | "1" => "1",
        "SIGINT" | "INT" | "2" => "2",
        "SIGQUIT" | "QUIT" | "3" => "3",
        "SIGSTOP" | "STOP" | "19" => "19",
        "SIGCONT" | "CONT" | "18" => "18",
        "SIGUSR1" | "USR1" | "10" => "10",
        "SIGUSR2" | "USR2" | "12" => "12",
        "SIGPIPE" | "PIPE" | "13" => "13",
        "SIGALRM" | "ALRM" | "14" => "14",
        "SIGTSTP" | "TSTP" | "20" => "20",
        "SIGTTIN" | "TTIN" | "21" => "21",
        "SIGTTOU" | "TTOU" | "22" => "22",
        _ => return Err(format!("Unknown signal '{}'", signal)),
    };

    let status = std::process::Command::new("kill")
        .arg(format!("-{}", sig))
        .arg(pid.to_string())
        .status()
        .map_err(|e| format!("Failed to execute kill: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        match status.code() {
            Some(1) => Err("Permission denied".into()),
            Some(64) => Err("Invalid signal".into()),
            Some(3) => Err("No such process".into()),
            _ => Err("Operation failed".into()),
        }
    }
}

#[cfg(windows)]
fn send_signal(pid: u32, _signal: &str) -> Result<(), String> {
    let status = std::process::Command::new("taskkill")
        .arg("/PID")
        .arg(pid.to_string())
        .arg("/F")
        .status()
        .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("Failed to terminate process".into())
    }
}

fn get_process_name(pid: u32) -> String {
    #[cfg(target_os = "linux")]
    {
        let path = format!("/proc/{}/comm", pid);
        std::fs::read_to_string(&path)
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".into())
    }
    #[cfg(not(target_os = "linux"))]
    {
        "unknown".into()
    }
}

fn validate_pid(s: &str) -> Option<u32> {
    let pid: u32 = s.parse().ok()?;
    if pid == 0 {
        return None;
    }
    #[cfg(unix)]
    {
        if Path::new(&format!("/proc/{}", pid)).exists() {
            Some(pid)
        } else {
            None
        }
    }
    #[cfg(not(unix))]
    {
        Some(pid)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let mut pid: Option<u32> = None;
    let mut signal = "TERM";

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Terminates or signals a process.");
                println!("  --signal, -s <sig>  Signal to send (default: TERM)");
                println!("                     Common signals: TERM(15), KILL(9), HUP(1)");
                println!("  --help, -h         Show this help message");
                println!("  --version          Show version information");
                return;
            }
            "--version" => {
                println!("kill version 0.1.0");
                return;
            }
            "--signal" | "-s" => {
                i += 1;
                if i < args.len() {
                    signal = &args[i];
                } else {
                    eprintln!("Error: --signal/-s requires a value");
                    std::process::exit(1);
                }
            }
            _ => {
                if pid.is_none() {
                    pid = validate_pid(&args[i]);
                }
            }
        }
        i += 1;
    }

    let pid = match pid {
        Some(p) => p,
        None => {
            print_usage();
            std::process::exit(1);
        }
    };

    let name = get_process_name(pid);
    println!("Sending signal {} to PID {} ({})", signal, pid, name);

    match send_signal(pid, signal) {
        Ok(_) => println!("Signal sent to PID {}", pid),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

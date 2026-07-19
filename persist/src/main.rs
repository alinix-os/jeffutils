use std::env;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::process::Command;

fn print_usage() {
    eprintln!("Usage: persist [OPTION]... COMMAND [ARG]...");
    eprintln!("Run a command immune to hangups.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -o FILE   output file (default: nohup.out or $OUTFILE)");
    eprintln!("  -h, --help       display this help and exit");
    eprintln!("  -v, --version    display version and exit");
}

fn is_terminal(fd: i32) -> bool {
    unsafe { libc::isatty(fd) != 0 }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("persist (JeffUtils) 1.0");
        return;
    }

    let mut output_file: Option<String> = None;
    let mut cmd_start = 0;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("persist: option '-o' requires an argument");
                    std::process::exit(1);
                }
                output_file = Some(args[i].clone());
            }
            _ => {
                cmd_start = i;
                break;
            }
        }
        i += 1;
    }

    if cmd_start >= args.len() {
        eprintln!("persist: missing COMMAND");
        std::process::exit(1);
    }

    let outfile_path = output_file.unwrap_or_else(|| {
        env::var("OUTFILE").unwrap_or_else(|_| "nohup.out".to_string())
    });

    // Install SIGHUP handler to ignore
    unsafe {
        let mut act: libc::sigaction = std::mem::zeroed();
        act.sa_sigaction = libc::SIG_IGN;
        libc::sigaction(libc::SIGHUP, &act, std::ptr::null_mut());
    }

    // Redirect stdout if it's a terminal
    if is_terminal(libc::STDOUT_FILENO) {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&outfile_path)
            .unwrap_or_else(|e| {
                eprintln!("persist: cannot open '{}': {}", outfile_path, e);
                std::process::exit(1);
            });
        unsafe {
            libc::dup2(file.as_raw_fd(), libc::STDOUT_FILENO);
        }
    }

    // Redirect stderr if it's a terminal
    if is_terminal(libc::STDERR_FILENO) {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&outfile_path)
            .unwrap_or_else(|e| {
                eprintln!("persist: cannot open '{}': {}", outfile_path, e);
                std::process::exit(1);
            });
        unsafe {
            libc::dup2(file.as_raw_fd(), libc::STDERR_FILENO);
        }
    }

    let cmd = &args[cmd_start];
    let cmd_args = &args[cmd_start + 1..];

    let mut child = match Command::new(cmd).args(cmd_args).spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("persist: failed to execute '{}': {}", cmd, e);
            std::process::exit(127);
        }
    };

    let status = child.wait().unwrap_or_else(|e| {
        eprintln!("persist: wait error: {}", e);
        std::process::exit(1);
    });

    std::process::exit(status.code().unwrap_or(0));
}

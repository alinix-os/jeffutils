use std::env;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("conrun - run command in a SELinux context");
    eprintln!();
    eprintln!("USAGE: conrun CONTEXT COMMAND [ARGS...]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
}

fn selinux_available() -> bool {
    std::path::Path::new("/sys/fs/selinux").exists()
        || std::path::Path::new("/proc/filesystems").exists()
            && std::fs::read_to_string("/proc/filesystems")
                .map(|s| s.contains("selinux"))
                .unwrap_or(false)
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("conrun", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    let mut positional: Vec<String> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("conrun {}", VERSION);
                process::exit(0);
            }
            _ => {
                positional.push(args[i].clone());
            }
        }
        i += 1;
    }

    if positional.len() < 2 {
        eprintln!("conrun: expected CONTEXT COMMAND [ARGS...]");
        print_usage();
        process::exit(2);
    }

    let _context = &positional[0];
    let command = &positional[1];
    let cmd_args = &positional[2..];

    if !selinux_available() {
        eprintln!("conrun: warning: SELinux not available, running command anyway");
    }

    let mut child = process::Command::new(command)
        .args(cmd_args)
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("conrun: cannot execute '{}': {}", command, e);
            process::exit(1);
        });

    let status = child.wait().unwrap_or_else(|e| {
        eprintln!("conrun: wait failed: {}", e);
        process::exit(1);
    });

    process::exit(status.code().unwrap_or(1));
}

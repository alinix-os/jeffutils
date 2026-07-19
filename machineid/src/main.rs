use std::env;
use std::fs;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("machineid - print unique host identifier");
    eprintln!();
    eprintln!("USAGE: machineid [OPTIONS]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  --uuid      read from /etc/machine-id instead");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut use_uuid = false;

    for arg in &args {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("machineid {}", VERSION);
                process::exit(0);
            }
            "--uuid" => use_uuid = true,
            _ => {
                eprintln!("machineid: unknown option '{}'", arg);
                process::exit(2);
            }
        }
    }

    if use_uuid {
        let content = fs::read_to_string("/etc/machine-id").unwrap_or_else(|e| {
            eprintln!("machineid: cannot read /etc/machine-id: {}", e);
            process::exit(1);
        });

        let id = content.trim();
        if id.is_empty() {
            eprintln!("machineid: /etc/machine-id is empty");
            process::exit(1);
        }
        println!("{}", id);
    } else {
        let hostid = unsafe { libc::gethostid() };
        println!("{:08x}", hostid as u32);
    }
}

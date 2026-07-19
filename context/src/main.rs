use std::env;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("context - print or change SELinux context");
    eprintln!();
    eprintln!("USAGE: context [-t TYPE] FILE...");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -t TYPE     set type component");
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
    let args: Vec<String> = env::args().skip(1).collect();

    let mut set_type: Option<String> = None;
    let mut files: Vec<String> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("context {}", VERSION);
                process::exit(0);
            }
            "-t" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("context: option -t requires an argument");
                    process::exit(2);
                }
                set_type = Some(args[i].clone());
            }
            _ if args[i].starts_with('-') => {
                eprintln!("context: unknown option '{}'", args[i]);
                process::exit(2);
            }
            _ => {
                files.push(args[i].clone());
            }
        }
        i += 1;
    }

    if !selinux_available() {
        eprintln!("context: SELinux not available on this system");
        if set_type.is_some() {
            process::exit(1);
        }
        process::exit(0);
    }

    if files.is_empty() {
        eprintln!("context: missing FILE operand");
        print_usage();
        process::exit(2);
    }

    for file in &files {
        if let Some(ref _type_val) = set_type {
            let output = process::Command::new("chcon")
                .arg("-t")
                .arg(_type_val)
                .arg(file)
                .output();

            match output {
                Ok(o) => {
                    if !o.status.success() {
                        eprintln!(
                            "context: chcon failed: {}",
                            String::from_utf8_lossy(&o.stderr)
                        );
                        process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("context: cannot run chcon: {}", e);
                    process::exit(1);
                }
            }
        } else {
            let output = process::Command::new("ls")
                .arg("-Z")
                .arg(file)
                .output();

            match output {
                Ok(o) => {
                    if o.status.success() {
                        print!("{}", String::from_utf8_lossy(&o.stdout));
                    } else {
                        eprintln!(
                            "context: ls -Z failed: {}",
                            String::from_utf8_lossy(&o.stderr)
                        );
                        process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("context: cannot run ls: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}

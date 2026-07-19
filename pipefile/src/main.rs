use std::env;
use std::ffi::CString;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("pipefile - create named pipe (FIFO)");
    eprintln!();
    eprintln!("USAGE: pipefile [-m MODE] NAME...");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -m MODE     permissions in octal (default: 0644)");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
}

fn parse_mode(s: &str) -> Result<u32, String> {
    u32::from_str_radix(s, 8).map_err(|_| format!("invalid octal mode: {}", s))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut mode: u32 = 0o644;
    let mut names: Vec<String> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("pipefile {}", VERSION);
                process::exit(0);
            }
            "-m" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("pipefile: option -m requires an argument");
                    process::exit(2);
                }
                mode = match parse_mode(&args[i]) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("pipefile: {}", e);
                        process::exit(2);
                    }
                };
            }
            _ if args[i].starts_with('-') => {
                eprintln!("pipefile: unknown option '{}'", args[i]);
                process::exit(2);
            }
            _ => {
                names.push(args[i].clone());
            }
        }
        i += 1;
    }

    if names.is_empty() {
        eprintln!("pipefile: missing NAME operand");
        print_usage();
        process::exit(2);
    }

    let mut exit_code = 0;

    for name in &names {
        let c_name = match CString::new(name.clone()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("pipefile: invalid path '{}': {}", name, e);
                exit_code = 1;
                continue;
            }
        };

        let ret = unsafe { libc::mkfifo(c_name.as_ptr(), mode) };
        if ret != 0 {
            let err = std::io::Error::last_os_error();
            eprintln!("pipefile: cannot create '{}': {}", name, err);
            exit_code = 1;
        }
    }

    process::exit(exit_code);
}

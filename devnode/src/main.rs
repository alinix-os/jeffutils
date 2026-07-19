use std::env;
use std::ffi::CString;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("devnode - create block or character special files");
    eprintln!();
    eprintln!("USAGE: devnode [-b|-c|-u] [-m MODE] NAME TYPE MAJOR MINOR");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -b          create a block device");
    eprintln!("  -c, -u      create a character device");
    eprintln!("  -m MODE     permissions in octal (default: 0660)");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
}

fn parse_mode(s: &str) -> Result<u32, String> {
    u32::from_str_radix(s, 8).map_err(|_| format!("invalid octal mode: {}", s))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut mode: u32 = 0o660;
    let mut is_block = false;
    let mut positional: Vec<String> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("devnode {}", VERSION);
                process::exit(0);
            }
            "-b" => is_block = true,
            "-c" | "-u" => is_block = false,
            "-m" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("devnode: option -m requires an argument");
                    process::exit(2);
                }
                mode = match parse_mode(&args[i]) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("devnode: {}", e);
                        process::exit(2);
                    }
                };
            }
            _ if args[i].starts_with('-') => {
                eprintln!("devnode: unknown option '{}'", args[i]);
                process::exit(2);
            }
            _ => {
                positional.push(args[i].clone());
            }
        }
        i += 1;
    }

    if positional.len() != 4 {
        eprintln!("devnode: expected NAME TYPE MAJOR MINOR");
        print_usage();
        process::exit(2);
    }

    let name = &positional[0];
    let _type_str = &positional[1];

    let major: u64 = positional[2].parse().unwrap_or_else(|_| {
        eprintln!("devnode: invalid major number '{}'", positional[2]);
        process::exit(2);
    });

    let minor: u64 = positional[3].parse().unwrap_or_else(|_| {
        eprintln!("devnode: invalid minor number '{}'", positional[3]);
        process::exit(2);
    });

    let c_name = match CString::new(name.clone()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("devnode: invalid path '{}': {}", name, e);
            process::exit(1);
        }
    };

    let dev = libc::makedev(major as u32, minor as u32);
    let filetype = if is_block {
        libc::S_IFBLK
    } else {
        libc::S_IFCHR
    };

    let ret = unsafe { libc::mknod(c_name.as_ptr(), filetype | mode, dev) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        eprintln!("devnode: cannot create '{}': {}", name, err);
        process::exit(1);
    }
}

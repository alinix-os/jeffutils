use std::env;
use std::ffi::CString;
use std::path::Path;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("jail - run command in modified root directory");
    eprintln!();
    eprintln!("USAGE: jail NEWROOT COMMAND [ARGS...]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("jail", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
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
                println!("jail {}", VERSION);
                process::exit(0);
            }
            _ => {
                positional.push(args[i].clone());
            }
        }
        i += 1;
    }

    if positional.len() < 2 {
        eprintln!("jail: expected NEWROOT COMMAND [ARGS...]");
        print_usage();
        process::exit(2);
    }

    let newroot = &positional[0];
    let command = &positional[1];
    let cmd_args = &positional[2..];

    if !Path::new(newroot).is_dir() {
        eprintln!("jail: '{}' is not a directory", newroot);
        process::exit(1);
    }

    let c_newroot = CString::new(newroot.clone()).unwrap_or_else(|e| {
        eprintln!("jail: invalid path '{}': {}", newroot, e);
        process::exit(1);
    });

    let c_slash = CString::new("/").unwrap();

    unsafe {
        if libc::chroot(c_newroot.as_ptr()) != 0 {
            let err = std::io::Error::last_os_error();
            eprintln!("jail: chroot failed: {}", err);
            process::exit(1);
        }

        if libc::chdir(c_slash.as_ptr()) != 0 {
            let err = std::io::Error::last_os_error();
            eprintln!("jail: chdir failed: {}", err);
            process::exit(1);
        }
    }

    let mut child = process::Command::new(command)
        .args(cmd_args)
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("jail: cannot execute '{}': {}", command, e);
            process::exit(1);
        });

    let status = child.wait().unwrap_or_else(|e| {
        eprintln!("jail: wait failed: {}", e);
        process::exit(1);
    });

    process::exit(status.code().unwrap_or(1));
}

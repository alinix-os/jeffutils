use std::collections::HashSet;
use std::env;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("sessions - list logged-in user names");
    eprintln!();
    eprintln!("USAGE: sessions [OPTIONS]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("sessions", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("sessions {}", VERSION);
                process::exit(0);
            }
            _ => {
                eprintln!("sessions: unknown option '{}'", arg);
                process::exit(2);
            }
        }
    }

    let mut seen = HashSet::new();

    unsafe {
        libc::setutxent();

        loop {
            let entry = libc::getutxent();
            if entry.is_null() {
                break;
            }

            let entry = &*entry;

            if entry.ut_type == libc::USER_PROCESS as libc::c_short
                || entry.ut_type == libc::LOGIN_PROCESS as libc::c_short
            {
                let name_bytes = &entry.ut_user;
                let name = std::ffi::CStr::from_ptr(name_bytes.as_ptr())
                    .to_string_lossy()
                    .trim()
                    .to_string();

                if !name.is_empty() && seen.insert(name.clone()) {
                    println!("{}", name);
                }
            }
        }

        libc::endutxent();
    }
}

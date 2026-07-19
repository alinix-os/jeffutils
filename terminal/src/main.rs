use std::env;

fn print_usage() {
    eprintln!("Usage: terminal [OPTION]...");
    eprintln!("Print the terminal device name.");
    eprintln!();
    eprintln!("Exit 0 if stdin is a terminal, 1 if not.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -s           silent mode (no output, just exit code)");
    eprintln!("  -h, --help       display this help and exit");
    eprintln!("  -v, --version    display version and exit");
}

fn get_tty_name() -> Option<String> {
    unsafe {
        let name = libc::ttyname(libc::STDIN_FILENO);
        if name.is_null() {
            None
        } else {
            Some(std::ffi::CStr::from_ptr(name).to_string_lossy().into_owned())
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("terminal (JeffUtils) 1.0");
        return;
    }

    let silent = args.iter().any(|a| a == "-s");

    let is_tty = unsafe { libc::isatty(libc::STDIN_FILENO) != 0 };

    if !is_tty {
        std::process::exit(1);
    }

    if let Some(name) = get_tty_name() {
        if !silent {
            println!("{}", name);
        }
        std::process::exit(0);
    }

    std::process::exit(1);
}

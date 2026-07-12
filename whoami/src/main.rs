fn print_usage() {
    eprintln!("Usage: {}", std::env::args().nth(0).unwrap_or_else(|| "whoami".into()));
}

fn get_username() -> String {
    #[cfg(unix)]
    {
        unsafe {
            let uid = libc::geteuid();
            let pwd = libc::getpwuid(uid);
            if !pwd.is_null() {
                if let Ok(name) = std::ffi::CStr::from_ptr((*pwd).pw_name).to_str() {
                    return name.to_string();
                }
            }
        }
    }

    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        if arg == "--help" || arg == "-h" {
            print_usage();
            println!("Prints the current user name.");
            println!("  --help, -h  Show this help message");
            println!("  --version   Show version information");
            return;
        }
        if arg == "--version" {
            println!("whoami version 0.1.0");
            return;
        }
    }

    if !args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    println!("{}", get_username());
}

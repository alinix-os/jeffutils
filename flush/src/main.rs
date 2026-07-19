fn print_usage() {
    eprintln!("Usage: flush [OPTION]...");
    eprintln!("Flush filesystem buffers to disk.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -d           sync open file descriptors (syncfs)");
    eprintln!("  -h, --help       display this help and exit");
    eprintln!("  -v, --version    display version and exit");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("flush (JeffUtils) 1.0");
        return;
    }

    let syncfs = args.iter().any(|a| a == "-d");

    if syncfs {
        #[cfg(target_os = "linux")]
        {
            unsafe {
                libc::syncfs(0);
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            unsafe {
                libc::sync();
            }
        }
    } else {
        unsafe {
            libc::sync();
        }
    }

    println!("filesystem buffers flushed");
}

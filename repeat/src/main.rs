use std::env;
use std::io::{self, Write};

const VERSION: &str = "0.1.0";

fn print_usage() {
    let name = env::args().next().unwrap_or_else(|| "repeat".into());
    eprintln!("Usage: {name} [STRING]...");
    eprintln!("Repeatedly print STRING until killed.");
    eprintln!("With no arguments, prints 'y' repeatedly.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help      show this help message");
    eprintln!("  -v, --version   show version");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "-v" || arg == "--version" {
            println!("repeat {VERSION}");
            return;
        }
    }

    let output = if args.is_empty() {
        "y".to_string()
    } else {
        args.join(" ")
    };

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    loop {
        writeln!(handle, "{output}").unwrap_or_else(|e| {
            if e.kind() == io::ErrorKind::BrokenPipe {
                std::process::exit(0);
            }
            eprintln!("repeat: write error: {e}");
            std::process::exit(1);
        });
        handle.flush().unwrap_or_else(|e| {
            if e.kind() == io::ErrorKind::BrokenPipe {
                std::process::exit(0);
            }
            eprintln!("repeat: flush error: {e}");
            std::process::exit(1);
        });
    }
}

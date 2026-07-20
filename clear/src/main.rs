use std::io::Write;

fn print_usage() {
    eprintln!("Usage: {}", std::env::args().nth(0).unwrap_or_else(|| "clear".into()));
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("clear", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        if arg == "--help" || arg == "-h" {
            print_usage();
            println!("Clears the terminal screen.");
            println!("  --help, -h  Show this help message");
            println!("  --version   Show version information");
            return;
        }
        if arg == "--version" {
            println!("clear version 0.1.0");
            return;
        }
    }

    if !args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    print!("\x1B[2J\x1B[3J\x1B[1;1H");
    std::io::stdout().flush().ok();
}

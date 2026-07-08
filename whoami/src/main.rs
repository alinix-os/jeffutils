fn print_usage() {
    eprintln!("Usage: {}", std::env::args().nth(0).unwrap_or_else(|| "whoami".into()));
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

    let user = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "unknown".into());
    println!("{}", user);
}

fn print_usage() {
    eprintln!("Usage: {}", std::env::args().nth(0).unwrap_or_else(|| "pwd".into()));
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        if arg == "--help" || arg == "-h" {
            print_usage();
            println!("Prints the current working directory.");
            println!("  --help, -h  Show this help message");
            println!("  --version   Show version information");
            return;
        }
        if arg == "--version" {
            println!("pwd version 0.1.0");
            return;
        }
    }

    if !args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    match std::env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

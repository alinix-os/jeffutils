use std::env;
use std::path::Path;

const VERSION: &str = "0.1.0";

fn print_usage() {
    let name = env::args().next().unwrap_or_else(|| "stem".into());
    eprintln!("Usage: {name} PATH");
    eprintln!("Print the directory component of PATH (everything before the last '/').");
    eprintln!();
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
            println!("stem {VERSION}");
            return;
        }
    }

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let path = Path::new(&args[0]);
    match path.parent() {
        Some(parent) => {
            if parent.as_os_str().is_empty() {
                println!(".");
            } else {
                println!("{}", parent.display());
            }
        }
        None => println!("."),
    }
}

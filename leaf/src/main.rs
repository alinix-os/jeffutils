use std::env;
use std::path::Path;

const VERSION: &str = "0.1.0";

fn print_usage() {
    let name = env::args().next().unwrap_or_else(|| "leaf".into());
    eprintln!("Usage: {name} PATH [SUFFIX]");
    eprintln!("Print the last component of PATH, optionally stripping SUFFIX.");
    eprintln!();
    eprintln!("  -h, --help      show this help message");
    eprintln!("  -v, --version   show version");
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("leaf", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "-v" || arg == "--version" {
            println!("leaf {VERSION}");
            return;
        }
    }

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let path = Path::new(&args[0]);
    let basename = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();

    if let Some(suffix) = args.get(1) {
        if let Some(stripped) = basename.strip_suffix(suffix) {
            println!("{stripped}");
        } else {
            println!("{basename}");
        }
    } else {
        println!("{basename}");
    }
}

use std::env;
use std::path::PathBuf;

const VERSION: &str = "0.1.0";

fn print_usage() {
    let name = env::args().next().unwrap_or_else(|| "resolve".into());
    eprintln!("Usage: {name} [-e] [-m] FILE...");
    eprintln!("Print the canonical absolute pathname, resolving symlinks, '.', and '..'.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -e          require each file to exist (default)");
    eprintln!("  -m          do not require file existence (create path mentally)");
    eprintln!("  -h, --help  show this help message");
    eprintln!("  -v, --version show version");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "-v" || arg == "--version" {
            println!("resolve {VERSION}");
            return;
        }
    }

    let mut require_existence = true;
    let mut files: Vec<&str> = Vec::new();

    for arg in &args {
        match arg.as_str() {
            "-e" => require_existence = true,
            "-m" => require_existence = false,
            other => files.push(other),
        }
    }

    if files.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let mut had_error = false;

    for file in &files {
        let path = PathBuf::from(file);

        if require_existence {
            match std::fs::canonicalize(&path) {
                Ok(canonical) => println!("{}", canonical.display()),
                Err(e) => {
                    eprintln!("resolve: {file}: {e}");
                    had_error = true;
                }
            }
        } else {
            // For -m: resolve as much as possible without requiring existence
            let mut result = PathBuf::new();
            for component in path.components() {
                use std::path::Component;
                match component {
                    Component::ParentDir => {
                        result.pop();
                    }
                    Component::CurDir => {}
                    Component::Normal(c) => result.push(c),
                    Component::RootDir => result.push("/"),
                    Component::Prefix(p) => result.push(p.as_os_str()),
                }
            }
            // Try to canonicalize if the path happens to exist
            if let Ok(canonical) = std::fs::canonicalize(&result) {
                println!("{}", canonical.display());
            } else {
                println!("{}", result.display());
            }
        }
    }

    if had_error {
        std::process::exit(1);
    }
}

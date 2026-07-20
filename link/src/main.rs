use std::fs;
use std::io::ErrorKind;
use std::path::Path;

fn describe_error(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "File not found",
        ErrorKind::PermissionDenied => "Do not have permissions",
        ErrorKind::AlreadyExists => "Link already exists",
        ErrorKind::ReadOnlyFilesystem => "Filesystem is read-only",
        ErrorKind::InvalidInput => "Invalid input",
        ErrorKind::Interrupted => "Operation interrupted",
        _ => "Unknown error occurred",
    }
}

fn print_usage() {
    eprintln!("Usage: {} <alvo> <link> [-s]", std::env::args().nth(0).unwrap_or_else(|| "link".into()));
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("link", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut symbolic = false;
    let mut paths = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Creates links between files.");
                println!("  -s, --symbolic  Create a symbolic link (default: hard link)");
                println!("  --help, -h      Show this help message");
                println!("  --version       Show version information");
                return;
            }
            "--version" => {
                println!("link version 0.1.0");
                return;
            }
            "-s" | "--symbolic" => symbolic = true,
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let target = Path::new(&paths[0]);
    let link_path = Path::new(&paths[1]);

    let result = if symbolic {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link_path)
        }
        #[cfg(windows)]
        {
            if target.is_dir() {
                std::os::windows::fs::symlink_dir(target, link_path)
            } else {
                std::os::windows::fs::symlink_file(target, link_path)
            }
        }
    } else {
        fs::hard_link(target, link_path)
    };

    match result {
        Ok(_) => println!("Created {} link: {} -> {}", if symbolic { "symbolic" } else { "hard" }, paths[1], paths[0]),
        Err(e) => {
            eprintln!("Error: {}", describe_error(e.kind()));
            std::process::exit(1);
        }
    }
}

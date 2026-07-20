use std::path::Path;

fn print_usage() {
    eprintln!("Usage: {} <caminho>", std::env::args().nth(0).unwrap_or_else(|| "fscheck".into()));
}

fn check_path(path: &Path) {
    if !path.exists() {
        println!("{}: Path does not exist", path.display());
        return;
    }

    let metadata = match path.metadata() {
        Ok(m) => m,
        Err(e) => {
            println!("{}: Cannot read metadata - {}", path.display(), e);
            return;
        }
    };

    println!("Path          : {}", path.display());
    println!("Type          : {}", if metadata.is_dir() { "Directory" } else if metadata.is_file() { "File" } else { "Other" });
    println!("Size          : {} bytes", metadata.len());

    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let mode = metadata.permissions().mode();
        println!("Permissions   : {:o}", mode & 0o7777);
        println!("Owner UID     : {}", metadata.uid());
        println!("Group GID     : {}", metadata.gid());
    }

    println!("Read-only     : {}", metadata.permissions().readonly());

    if let Ok(modified) = metadata.modified() {
        if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
            println!("Modified      : {}s since epoch", duration.as_secs());
        }
    }

    if let Ok(created) = metadata.created() {
        if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
            println!("Created       : {}s since epoch", duration.as_secs());
        }
    }

    if metadata.is_dir() {
        match std::fs::read_dir(path) {
            Ok(entries) => {
                let count = entries.flatten().count();
                println!("Entries       : {}", count);
            }
            Err(e) => {
                println!("Read error    : {}", e);
            }
        }
    }

    println!("Status        : OK");
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("fscheck", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut path = ".";

    for arg in &args {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Checks filesystem integrity and shows metadata.");
                println!("  --help, -h  Show this help message");
                println!("  --version   Show version information");
                return;
            }
            "--version" => {
                println!("fscheck version 0.1.0");
                return;
            }
            _ => {
                if path == "." {
                    path = arg;
                }
            }
        }
    }

    check_path(Path::new(path));
}

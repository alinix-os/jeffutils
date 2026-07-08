use std::fs;
use std::io::ErrorKind;
use std::path::Path;

fn describe_error(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "File not found",
        ErrorKind::PermissionDenied => "Do not have permissions",
        ErrorKind::AlreadyExists => "Destination already exists",
        ErrorKind::ReadOnlyFilesystem => "Filesystem is read-only",
        ErrorKind::ResourceBusy => "Resource busy",
        ErrorKind::CrossesDevices => "Operation crosses devices",
        ErrorKind::InvalidInput => "Invalid input",
        ErrorKind::Interrupted => "Operation interrupted",
        ErrorKind::Unsupported => "Operation not supported",
        _ => "Unknown error occurred",
    }
}

fn move_item(src: &Path, dst: &Path) -> Result<(), String> {
    fs::rename(src, dst).or_else(|e| {
        if e.kind() == ErrorKind::CrossesDevices {
            if src.is_dir() {
                return Err("Cannot move directory across devices. Use copy + remove instead.".into());
            }
            fs::copy(src, dst).and_then(|_| fs::remove_file(src)).map_err(|e| describe_error(e.kind()).into())
        } else {
            Err(describe_error(e.kind()).into())
        }
    })
}

fn print_usage() {
    eprintln!("Usage: {} <origem> <destino>", std::env::args().nth(0).unwrap_or_else(|| "mv".into()));
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut paths = Vec::new();

    for arg in &args {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Moves files or directories.");
                println!("  --help, -h  Show this help message");
                println!("  --version   Show version information");
                return;
            }
            "--version" => {
                println!("mv version 0.1.0");
                return;
            }
            _ => paths.push(arg.clone()),
        }
    }

    if paths.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let src = Path::new(&paths[0]);
    let dst = Path::new(&paths[1]);

    if dst.is_dir() {
        let filename = src.file_name().unwrap_or_default();
        let dst_path = dst.join(filename);
        match move_item(src, &dst_path) {
            Ok(_) => println!("Moved {} to {}", paths[0], dst_path.display()),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        match move_item(src, dst) {
            Ok(_) => println!("Moved {} to {}", paths[0], paths[1]),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

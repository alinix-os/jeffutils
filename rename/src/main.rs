use std::fs;
use std::io::ErrorKind;
use std::path::Path;

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn describe_error(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "File not found",
        ErrorKind::PermissionDenied => "Do not have permissions",
        ErrorKind::AlreadyExists => "Destination already exists",
        ErrorKind::ReadOnlyFilesystem => "Filesystem is read-only",
        ErrorKind::ResourceBusy => "Resource busy",
        ErrorKind::InvalidInput => "Invalid input",
        ErrorKind::Interrupted => "Operation interrupted",
        ErrorKind::CrossesDevices => "Cannot rename across different filesystems",
        _ => "Unknown error occurred",
    }
}

fn print_usage() {
    eprintln!("Usage: {} <atual> <novo>", std::env::args().nth(0).unwrap_or_else(|| "rename".into()));
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut paths = Vec::new();

    for arg in &args {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Renames a file or directory.");
                println!("  --help, -h  Show this help message");
                println!("  --version   Show version information");
                return;
            }
            "--version" => {
                println!("rename version 0.1.0");
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

    match fs::rename(src, dst) {
        Ok(_) => println!("Renamed {} to {}", paths[0], paths[1]),
        Err(e) if e.kind() == ErrorKind::CrossesDevices => {
            if src.is_dir() {
                if let Err(e) = copy_dir_recursive(src, dst) {
                    eprintln!("Error: {}", describe_error(e.kind()));
                    std::process::exit(1);
                }
                if let Err(e) = fs::remove_dir_all(src) {
                    eprintln!("Error: {}", describe_error(e.kind()));
                    std::process::exit(1);
                }
            } else {
                if let Err(e) = fs::copy(src, dst) {
                    eprintln!("Error: {}", describe_error(e.kind()));
                    std::process::exit(1);
                }
                if let Err(e) = fs::remove_file(src) {
                    eprintln!("Error: {}", describe_error(e.kind()));
                    std::process::exit(1);
                }
            }
            println!("Renamed {} to {}", paths[0], paths[1]);
        }
        Err(e) => {
            eprintln!("Error: {}", describe_error(e.kind()));
            std::process::exit(1);
        }
    }
}

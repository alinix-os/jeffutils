use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

fn describe_error(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "File not found",
        ErrorKind::PermissionDenied => "Do not have permissions",
        ErrorKind::AlreadyExists => "Destination already exists",
        ErrorKind::ReadOnlyFilesystem => "Filesystem is read-only",
        ErrorKind::StorageFull => "Storage full",
        ErrorKind::QuotaExceeded => "Quota exceeded",
        ErrorKind::FileTooLarge => "File too large",
        ErrorKind::ResourceBusy => "Resource busy",
        ErrorKind::InvalidInput => "Invalid input",
        ErrorKind::Interrupted => "Operation interrupted",
        ErrorKind::Unsupported => "Operation not supported",
        ErrorKind::OutOfMemory => "Out of memory",
        _ => "Unknown error occurred",
    }
}

fn copy_file(src: &Path, dst: &Path) -> io::Result<u64> {
    let mut input = fs::File::open(src)?;
    let mut output = fs::File::create(dst)?;
    if cfg!(unix) {
        if let Ok(perms) = fs::metadata(src).map(|m| m.permissions()) {
            let _ = fs::set_permissions(dst, perms);
        }
    }
    io::copy(&mut input, &mut output)
}

fn copy_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let entry_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if entry_type.is_dir() {
                copy_recursive(&src_path, &dst_path)?;
            } else {
                copy_file(&src_path, &dst_path)?;
            }
        }
    } else {
        copy_file(src, dst)?;
    }
    Ok(())
}

fn print_usage() {
    eprintln!("Usage: {} <origem> <destino> [-r]", std::env::args().nth(0).unwrap_or_else(|| "cp".into()));
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut recursive = false;
    let mut paths = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Copies files or directories.");
                println!("  -r, --recursive  Copy directories recursively");
                println!("  --help, -h       Show this help message");
                println!("  --version        Show version information");
                return;
            }
            "--version" => {
                println!("cp version 0.1.0");
                return;
            }
            "-r" | "--recursive" => recursive = true,
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let src = Path::new(&paths[0]);
    let dst = Path::new(&paths[1]);

    let result = if src.is_dir() && recursive {
        copy_recursive(src, dst)
    } else if src.is_dir() {
        Err(io::Error::new(ErrorKind::IsADirectory, "Source is a directory, use -r to copy recursively"))
    } else {
        if dst.is_dir() {
            let filename = src.file_name().unwrap_or_default();
            copy_file(src, &dst.join(filename))
        } else {
            copy_file(src, dst)
        }
        .map(|_| ())
    };

    match result {
        Ok(_) => println!("Copied {} to {}", paths[0], paths[1]),
        Err(e) => {
            eprintln!("Error: {}", describe_error(e.kind()));
            std::process::exit(1);
        }
    }
}

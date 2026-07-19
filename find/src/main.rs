use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

fn describe_error(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::PermissionDenied => "Permission denied",
        ErrorKind::NotFound => "Directory not found",
        ErrorKind::Interrupted => "Operation interrupted",
        _ => "Unknown error",
    }
}

fn find_files<P: AsRef<Path>>(dir: P, pattern: &str, exact: bool, max_depth: Option<usize>, case_sensitive: bool, current_depth: usize, visited: &mut HashSet<(u64, u64)>) {
    if let Some(max) = max_depth {
        if current_depth > max {
            return;
        }
    }

    let dir = dir.as_ref();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading {}: {}", dir.display(), describe_error(e.kind()));
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        let matches = if exact {
            if case_sensitive {
                name_str == pattern
            } else {
                name_str.eq_ignore_ascii_case(pattern)
            }
        } else {
            if case_sensitive {
                name_str.contains(pattern)
            } else {
                name_str.to_lowercase().contains(&pattern.to_lowercase())
            }
        };

        if matches {
            println!("{}", path.display());
        }

        if path.is_dir() {
            if let Ok(meta) = path.metadata() {
                let key = (meta.dev(), meta.ino());
                if !visited.insert(key) {
                    continue;
                }
            }
            find_files(&path, pattern, exact, max_depth, case_sensitive, current_depth + 1, visited);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: {} <padrão> [caminho] [opções]", std::env::args().nth(0).unwrap_or_else(|| "find".into()));
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let mut pattern = String::new();
    let mut path = ".".to_string();
    let mut exact = false;
    let mut max_depth: Option<usize> = None;
    let mut case_sensitive = true;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Locates files matching a pattern.");
                println!("  -e, --exact         Exact match (default: partial)");
                println!("  -i, --ignore-case   Case-insensitive search");
                println!("  -d, --max-depth <n> Maximum recursion depth");
                println!("  --help, -h          Show this help message");
                println!("  --version           Show version information");
                return;
            }
            "--version" => {
                println!("find version 0.1.0");
                return;
            }
            "-e" | "--exact" => exact = true,
            "-i" | "--ignore-case" => case_sensitive = false,
            "-d" | "--max-depth" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --max-depth requires a value");
                    std::process::exit(1);
                }
                max_depth = args[i].parse().ok();
            }
            _ => {
                if pattern.is_empty() {
                    pattern = args[i].clone();
                } else if path == "." {
                    path = args[i].clone();
                }
            }
        }
        i += 1;
    }

    if pattern.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let path = Path::new(&path);
    if !path.exists() {
        eprintln!("Error: path '{}' not found", path.display());
        std::process::exit(1);
    }

    find_files(path, &pattern, exact, max_depth, case_sensitive, 0, &mut HashSet::new());
}

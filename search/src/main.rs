use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

fn search_in_file(path: &Path, pattern: &str, case_sensitive: bool, show_filename: bool) {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return,
    };
    if metadata.len() > 10 * 1024 * 1024 {
        eprintln!("Warning: skipping {} (file too large: {} bytes)", path.display(), metadata.len());
        return;
    }
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let reader = BufReader::new(file);
    for (line_num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let found = if case_sensitive {
            line.contains(pattern)
        } else {
            line.to_lowercase().contains(&pattern.to_lowercase())
        };

        if found {
            if show_filename {
                println!("{}:{}:{}", path.display(), line_num + 1, line);
            } else {
                println!("{}:{}", line_num + 1, line);
            }
        }
    }
}

fn search_recursive<P: AsRef<Path>>(dir: P, pattern: &str, case_sensitive: bool, max_depth: Option<usize>, depth: usize, visited: &mut HashSet<(u64, u64)>) {
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }

    let dir = dir.as_ref();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Ok(meta) = entry.metadata() {
                let key = (meta.dev(), meta.ino());
                if !visited.insert(key) {
                    continue;
                }
            }
            search_recursive(&path, pattern, case_sensitive, max_depth, depth + 1, visited);
        } else if path.is_file() {
            search_in_file(&path, pattern, case_sensitive, true);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: {} <padrão> [caminho] [opções]", std::env::args().nth(0).unwrap_or_else(|| "search".into()));
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let mut pattern = String::new();
    let mut path = ".".to_string();
    let mut case_sensitive = true;
    let mut max_depth: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Searches for content in files.");
                println!("  -i, --ignore-case    Case-insensitive search");
                println!("  -d, --max-depth <n>  Maximum recursion depth");
                println!("  --help, -h           Show this help message");
                println!("  --version            Show version information");
                return;
            }
            "--version" => {
                println!("search version 0.1.0");
                return;
            }
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
    if path.is_file() {
        search_in_file(path, &pattern, case_sensitive, false);
    } else if path.is_dir() {
        search_recursive(path, &pattern, case_sensitive, max_depth, 0, &mut HashSet::new());
    } else {
        eprintln!("Error: path '{}' not found", path.display());
        std::process::exit(1);
    }
}

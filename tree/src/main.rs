use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

fn print_usage() {
    eprintln!("Usage: {} [caminho] [-L <nível>]", std::env::args().nth(0).unwrap_or_else(|| "tree".into()));
}

fn walk_dir(path: &Path, prefix: &str, max_depth: Option<usize>, depth: usize, visited: &mut HashSet<(u64, u64)>) {
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }

    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut files: Vec<_> = entries.flatten().collect();
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for (i, entry) in files.iter().enumerate() {
        let is_last_entry = i == files.len() - 1;
        let connector = if is_last_entry { "└── " } else { "├── " };

        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        println!("{}{}{}", prefix, connector, name_str);

        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            if let Ok(meta) = entry.metadata() {
                let key = (meta.dev(), meta.ino());
                if !visited.insert(key) {
                    continue;
                }
            }
            let next_prefix = format!("{}{}", prefix, if is_last_entry { "    " } else { "│   " });
            walk_dir(&entry.path(), &next_prefix, max_depth, depth + 1, visited);
        }
    }
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("tree", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut root = ".".to_string();
    let mut max_depth: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Displays a directory tree.");
                println!("  -L, --max-depth <n>  Limit recursion depth");
                println!("  --help, -h           Show this help message");
                println!("  --version            Show version information");
                return;
            }
            "--version" => {
                println!("tree version 0.2.0");
                return;
            }
            "-L" | "--max-depth" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --max-depth requires a value");
                    std::process::exit(1);
                }
                max_depth = args[i].parse().ok();
            }
            _ => root = args[i].clone(),
        }
        i += 1;
    }

    let path = Path::new(&root);
    if !path.exists() {
        eprintln!("Error: path '{}' not found", root);
        std::process::exit(1);
    }

    println!("{}", root);
    walk_dir(path, "", max_depth, 1, &mut HashSet::new());
}

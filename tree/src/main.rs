use std::path::Path;

fn print_usage() {
    eprintln!("Usage: {} [caminho] [-L <nível>]", std::env::args().nth(0).unwrap_or_else(|| "tree".into()));
}

fn walk_dir(path: &Path, prefix: &str, max_depth: Option<usize>, depth: usize, _is_last: bool) {
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
        let new_prefix = if is_last_entry { "    " } else { "│   " };

        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            println!("{}{}{}", new_prefix, connector, name_str);
            walk_dir(&entry.path(), &format!("{}{}", prefix, new_prefix), max_depth, depth + 1, is_last_entry);
        } else {
            println!("{}{}{}", prefix, connector, name_str);
        }
    }
}

fn main() {
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
                println!("tree version 0.1.0");
                return;
            }
            "-L" | "--max-depth" => {
                i += 1;
                if i < args.len() {
                    max_depth = args[i].parse().ok();
                }
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
    walk_dir(path, "", max_depth, 1, true);
}

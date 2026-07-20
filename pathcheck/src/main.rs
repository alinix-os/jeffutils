use std::env;
use std::path::Path;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("pathcheck - check pathname validity");
    eprintln!();
    eprintln!("USAGE: pathcheck [-p] [-e] PATH...");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -p          check all components exist as directories");
    eprintln!("  -e          check that the file exists");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
}

fn check_path(path: &str, check_parents: bool, check_exists: bool) -> Result<(), String> {
    if path.is_empty() {
        return Err("empty path name".to_string());
    }

    let parts: Vec<&str> = path.split('/').collect();

    for part in &parts {
        if part.is_empty() {
            continue;
        }
        if part.contains('/') {
            return Err(format!("name contains '/': {}", part));
        }
        if part.len() > 255 {
            return Err(format!("name too long ({} bytes): {}", part.len(), part));
        }
        for ch in part.chars() {
            if ch == '\0' {
                return Err("name contains null byte".to_string());
            }
        }
    }

    if check_parents {
        let p = Path::new(path);
        let mut current = p;
        let mut components: Vec<&Path> = Vec::new();

        while let Some(parent) = current.parent() {
            components.push(parent);
            current = parent;
        }

        for comp in components.iter().rev() {
            if comp.as_os_str().is_empty() {
                continue;
            }
            if !comp.is_dir() {
                return Err(format!("not a directory: {}", comp.display()));
            }
        }
    }

    if check_exists {
        let p = Path::new(path);
        if !p.exists() {
            return Err(format!("does not exist: {}", path));
        }
    }

    Ok(())
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("pathcheck", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    let mut check_parents = false;
    let mut check_exists = false;
    let mut paths: Vec<String> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("pathcheck {}", VERSION);
                process::exit(0);
            }
            "-p" => check_parents = true,
            "-e" => check_exists = true,
            _ if args[i].starts_with('-') => {
                eprintln!("pathcheck: unknown option '{}'", args[i]);
                process::exit(2);
            }
            _ => {
                paths.push(args[i].clone());
            }
        }
        i += 1;
    }

    if paths.is_empty() {
        eprintln!("pathcheck: missing PATH operand");
        print_usage();
        process::exit(2);
    }

    let mut exit_code = 0;

    for path in &paths {
        match check_path(path, check_parents, check_exists) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("pathcheck: {}: {}", path, e);
                exit_code = 1;
            }
        }
    }

    process::exit(exit_code);
}

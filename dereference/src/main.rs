use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const VERSION: &str = "0.1.0";

fn print_usage() {
    let name = env::args().next().unwrap_or_else(|| "dereference".into());
    eprintln!("Usage: {name} [-f] [-n] [-e] FILE");
    eprintln!("Print the value of a symbolic link, or resolve the full chain.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -f          canonicalize: follow entire chain and print resolved path");
    eprintln!("  -n          do not output trailing newline");
    eprintln!("  -e          resolve and print full canonical path (like readlink -f)");
    eprintln!("  -h, --help  show this help message");
    eprintln!("  -v, --version show version");
}

fn resolve_chain(path: &Path) -> Result<PathBuf, String> {
    let mut current = path.to_path_buf();
    let mut seen = std::collections::HashSet::new();

    loop {
        let canonical = fs::canonicalize(&current).map_err(|e| format!("{}: {e}", current.display()))?;
        if !seen.insert(canonical.clone()) {
            return Err(format!("{}: symlink loop detected", path.display()));
        }
        match fs::read_link(&current) {
            Ok(target) => {
                if target.is_absolute() {
                    current = target;
                } else {
                    if let Some(parent) = current.parent() {
                        current = parent.join(target);
                    } else {
                        current = target;
                    }
                }
            }
            Err(_) => return Ok(canonical),
        }
    }
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("dereference", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let raw_args: Vec<String> = env::args().skip(1).collect();

    for arg in &raw_args {
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "-v" || arg == "--version" {
            println!("dereference {VERSION}");
            return;
        }
    }

    let mut follow_all = false;
    let mut no_newline = false;
    let mut canonicalize = false;
    let mut files: Vec<&str> = Vec::new();

    for arg in &raw_args {
        match arg.as_str() {
            "-f" => follow_all = true,
            "-n" => no_newline = true,
            "-e" => canonicalize = true,
            other => files.push(other),
        }
    }

    if files.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let mut had_error = false;
    let mut output = String::new();

    for file in &files {
        let path = Path::new(file);

        let result = if follow_all || canonicalize {
            match resolve_chain(path) {
                Ok(resolved) => resolved.display().to_string(),
                Err(e) => {
                    eprintln!("dereference: {e}");
                    had_error = true;
                    continue;
                }
            }
        } else {
            match fs::read_link(path) {
                Ok(target) => target.display().to_string(),
                Err(e) => {
                    eprintln!("dereference: {file}: {e}");
                    had_error = true;
                    continue;
                }
            }
        };

        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(&result);
    }

    if no_newline {
        print!("{output}");
    } else {
        println!("{output}");
    }

    if had_error {
        std::process::exit(1);
    }
}

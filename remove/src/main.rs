
pub mod dir;
pub mod file;
pub mod safety;

use crate::safety::Risk;

use std::io::Write;
use std::str::FromStr;

use crate::file::remove as remove_file;

#[derive(Debug, PartialEq)]
enum ARGS {
    FILE,        // --file / -f
    DIR,         // --dir / -d
    RECURSIVE,   // --recursive / -r
    FORCE,       // --force
    HELP,        // --help / -h
    TARGET(String), // <path>
}

impl FromStr for ARGS {
    type Err = String;

    fn from_str(s: &str) -> Result<ARGS, String> {
        match s {
            "--file" | "-f" => Ok(ARGS::FILE),
            "--dir" | "-d" => Ok(ARGS::DIR),
            "--recursive" | "-r" => Ok(ARGS::RECURSIVE),
            "--force" => Ok(ARGS::FORCE),
            "--help" | "-h" => Ok(ARGS::HELP),
            _ => Ok(ARGS::TARGET(s.to_string())),
        }
    }
}

fn print_usage() {
    eprintln!("Usage: {} <destino> [-f|-d] [-r] [--force]", std::env::args().nth(0).unwrap_or_else(|| "command".into()));
}

fn confirm(prompt: &str) -> bool {
    print!("{} [y/N] ", prompt);
    std::io::stdout().flush().ok();

    let mut answer = String::new();
    if std::io::stdin().read_line(&mut answer).is_err() {
        return false;
    }

    matches!(answer.trim().to_lowercase().as_str(), "y" | "yes")
}

fn main() {
    let argv: Vec<ARGS> = std::env::args().collect::<Vec<String>>()[1..].iter().filter_map(|s| ARGS::from_str(s).ok()).collect();

    if argv.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    if argv.contains(&ARGS::HELP) {
        print_usage();
        println!("Options:");
        println!("  --file, -f         Remove a file (default)");
        println!("  --dir, -d          Remove a directory");
        println!("  --recursive, -r    Remove directories and their contents recursively");
        println!("  --force            Skip confirmation prompts");
        println!("  --help, -h         Show this help message");
        std::process::exit(0);
    }

    if argv.contains(&ARGS::FILE) && argv.contains(&ARGS::DIR) {
        eprintln!("Error: cannot use --file/-f and --dir/-d together");
        std::process::exit(1);
    }

    let path = match argv.iter().find_map(|a| match a {
        ARGS::TARGET(text) => Some(text.clone()),
        _ => None,
    }) {
        Some(path) => path,
        None => {
            print_usage();
            std::process::exit(1);
        }
    };

    let recursive = argv.contains(&ARGS::RECURSIVE);
    let force = argv.contains(&ARGS::FORCE);
    let is_dir = argv.contains(&ARGS::DIR);

    match safety::assess(&path) {
        Risk::Forbidden(reason) => {
            eprintln!("\x1b[1;31mrefused:\x1b[0m {}", reason);
            std::process::exit(1);
        }
        Risk::Critical(resolved) => {
            // Extremely dangerous target: always require the exact-path
            // confirmation, regardless of --force.
            if !safety::confirm_exact(&resolved) {
                println!("Aborted");
                std::process::exit(0);
            }
        }
        Risk::Normal => {
            if !force {
                let is_actually_dir = std::path::Path::new(&path).is_dir();
                let prompt = if is_dir || is_actually_dir {
                    format!("Remove directory '{}'{}?", path, if recursive { " and its contents" } else { "" })
                } else {
                    format!("Remove file '{}'?", path)
                };

                if !confirm(&prompt) {
                    println!("Aborted");
                    std::process::exit(0);
                }
            }
        }
    }

    if is_dir {
        dir::remove(&path, recursive);
    } else {
        remove_file(&path);
    }
}

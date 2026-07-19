use std::env;
use std::fs;
use std::path::Path;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Options {
    summarize: bool,
    all_files: bool,
    max_depth: Option<usize>,
    separate: bool,
    targets: Vec<String>,
}

fn print_usage() {
    println!("spaceused {} - estimate file space usage", VERSION);
    println!();
    println!("Usage: spaceused [OPTIONS] FILE...");
    println!();
    println!("Options:");
    println!("  -h, --help       display this help message");
    println!("  -v, --version    display version");
    println!("  -s               summarize (display only a total for each argument)");
    println!("  -h               human-readable sizes (K, M, G)");
    println!("  -a               include hidden files");
    println!("  -d DEPTH         max depth of directory recursion");
    println!("  -S               separate sizes for directories");
}

fn parse_args() -> Options {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut opts = Options {
        summarize: false,
        all_files: false,
        max_depth: None,
        separate: false,
        targets: Vec::new(),
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "-v" | "--version" => {
                println!("spaceused {}", VERSION);
                std::process::exit(0);
            }
            "-s" => opts.summarize = true,
            "-a" => opts.all_files = true,
            "-S" => opts.separate = true,
            "-d" => {
                i += 1;
                if i < args.len() {
                    opts.max_depth = args[i].parse().ok();
                } else {
                    eprintln!("spaceused: option requires an argument -- 'd'");
                    std::process::exit(1);
                }
            }
            other => {
                opts.targets.push(other.to_string());
            }
        }
        i += 1;
    }

    if opts.targets.is_empty() {
        opts.targets.push(".".to_string());
    }

    opts
}

fn human_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}G", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1}M", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{}", bytes)
    }
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

fn dir_size(path: &Path, opts: &Options, depth: usize) -> u64 {
    let max = opts.max_depth.unwrap_or(usize::MAX);
    let mut total: u64 = 0;

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if !opts.all_files && is_hidden(&name_str) {
            continue;
        }

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.is_file() {
            total += meta.len();
        } else if meta.is_dir() && depth < max {
            total += dir_size(&entry.path(), opts, depth + 1);
        }
    }

    total
}

fn main() {
    let opts = parse_args();

    let mut grand_total: u64 = 0;

    for target in &opts.targets {
        let path = Path::new(target);
        match fs::metadata(path) {
            Ok(meta) => {
                if meta.is_file() {
                    let size = meta.len();
                    grand_total += size;
                    if !opts.summarize {
                        println!("{}\t{}", human_size(size), target);
                    }
                } else if meta.is_dir() {
                    let size = dir_size(path, &opts, 0);
                    grand_total += size;
                    if !opts.summarize {
                        println!("{}\t{}", human_size(size), target);
                    }
                }
            }
            Err(e) => {
                eprintln!("spaceused: cannot access '{}': {}", target, e);
            }
        }
    }

    if opts.summarize || opts.targets.len() > 1 {
        println!("{}\ttotal", human_size(grand_total));
    }
}

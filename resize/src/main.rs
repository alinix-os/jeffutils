use std::env;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Options {
    size: Option<u64>,
    no_create: bool,
    files: Vec<String>,
}

fn print_usage() {
    println!("resize {} - shrink or extend files to a size", VERSION);
    println!();
    println!("Usage: resize [OPTIONS] FILE...");
    println!();
    println!("Options:");
    println!("  -h, --help       display this help message");
    println!("  -v, --version    display version");
    println!("  -s SIZE          set size (supports K/M/G suffixes)");
    println!("  -c               do not create files (fail if file does not exist)");
}

fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if let Some(val) = s.strip_suffix("K").or_else(|| s.strip_suffix("k")) {
        val.parse::<u64>().map(|v| v * 1024).map_err(|e| e.to_string())
    } else if let Some(val) = s.strip_suffix("M") {
        val.parse::<u64>().map(|v| v * 1_048_576).map_err(|e| e.to_string())
    } else if let Some(val) = s.strip_suffix("G") {
        val.parse::<u64>().map(|v| v * 1_073_741_824).map_err(|e| e.to_string())
    } else {
        s.parse::<u64>().map_err(|e| e.to_string())
    }
}

fn parse_args() -> Options {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut opts = Options {
        size: None,
        no_create: false,
        files: Vec::new(),
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("resize {}", VERSION);
                process::exit(0);
            }
            "-s" => {
                i += 1;
                if i < args.len() {
                    opts.size = Some(parse_size(&args[i]).unwrap_or_else(|e| {
                        eprintln!("resize: invalid size '{}': {}", args[i], e);
                        process::exit(1);
                    }));
                } else {
                    eprintln!("resize: option requires an argument -- 's'");
                    process::exit(1);
                }
            }
            "-c" => opts.no_create = true,
            other => {
                opts.files.push(other.to_string());
            }
        }
        i += 1;
    }

    if opts.size.is_none() {
        eprintln!("resize: -s SIZE is required");
        print_usage();
        process::exit(1);
    }

    if opts.files.is_empty() {
        eprintln!("resize: missing file operand");
        print_usage();
        process::exit(1);
    }

    opts
}

fn resize_file(path: &str, target_size: u64, no_create: bool) {
    let file_result = if no_create {
        OpenOptions::new().write(true).open(path)
    } else {
        OpenOptions::new().write(true).create(true).open(path)
    };

    let mut file = file_result.unwrap_or_else(|e| {
        eprintln!("resize: cannot open '{}': {}", path, e);
        process::exit(1);
    });

    let current_size = file.metadata().map(|m| m.len()).unwrap_or(0);

    if current_size > target_size {
        file.set_len(target_size).unwrap_or_else(|e| {
            eprintln!("resize: cannot truncate '{}': {}", path, e);
            process::exit(1);
        });
    } else if current_size < target_size {
        file.seek(SeekFrom::Start(target_size - 1)).unwrap_or_else(|e| {
            eprintln!("resize: cannot seek '{}': {}", path, e);
            process::exit(1);
        });
        file.write_all(&[0]).unwrap_or_else(|e| {
            eprintln!("resize: cannot extend '{}': {}", path, e);
            process::exit(1);
        });
    }
}

fn main() {
    let opts = parse_args();
    let target_size = opts.size.unwrap();

    for file in &opts.files {
        resize_file(file, target_size, opts.no_create);
    }
}

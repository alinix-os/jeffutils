use std::io::{self, Read, Write};
use std::process::exit;

const VERSION: &str = "0.1.0";

fn print_help() {
    eprintln!("mirror - read from stdin and write to stdout and file(s)");
    eprintln!("Usage: mirror [OPTIONS] FILE...");
    eprintln!("Read stdin, write to stdout and tee to file(s).");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -a   append to files instead of overwriting");
    eprintln!("  -i   ignore interrupt signals (SIGINT)");
    eprintln!("  -h   print this help");
    eprintln!("  -v   print version");
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("mirror", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut append_mode = false;
    let mut ignore_signals = false;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" => append_mode = true,
            "-i" => ignore_signals = true,
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            "-v" | "--version" => {
                eprintln!("mirror {}", VERSION);
                exit(0);
            }
            "--" => {
                i += 1;
                while i < args.len() {
                    files.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            a if a.starts_with('-') && a.len() > 1 => {
                for ch in a[1..].chars() {
                    match ch {
                        'a' => append_mode = true,
                        'i' => ignore_signals = true,
                        'h' => {
                            print_help();
                            exit(0);
                        }
                        'v' => {
                            eprintln!("mirror {}", VERSION);
                            exit(0);
                        }
                        _ => {
                            eprintln!("mirror: unknown option '-{}'", ch);
                            exit(1);
                        }
                    }
                }
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    if files.is_empty() {
        eprintln!("mirror: missing file operand");
        eprintln!("Try 'mirror -h' for more information.");
        exit(1);
    }

    if ignore_signals {
        unsafe {
            libc::signal(libc::SIGINT, libc::SIG_IGN);
        }
    }

    let mut open_files: Vec<std::fs::File> = files
        .iter()
        .map(|path| {
            let opts = if append_mode {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
            } else {
                std::fs::File::create(path)
            };
            opts.unwrap_or_else(|e| {
                eprintln!("mirror: {}: {}", path, e);
                exit(1);
            })
        })
        .collect();

    let mut stdin = io::stdin();
    let mut buffer = [0u8; 8192];
    let stdout = io::stdout();

    loop {
        let n = match stdin.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                if e.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                eprintln!("mirror: read error: {}", e);
                exit(1);
            }
        };

        {
            let mut out = stdout.lock();
            if out.write_all(&buffer[..n]).is_err() {
                eprintln!("mirror: write error to stdout");
                exit(1);
            }
        }

        for (idx, file) in open_files.iter_mut().enumerate() {
            if file.write_all(&buffer[..n]).is_err() {
                eprintln!("mirror: write error to {}", files[idx]);
                exit(1);
            }
        }
    }
}

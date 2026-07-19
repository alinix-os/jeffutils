use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Options {
    passes: usize,
    zero_final: bool,
    unlink: bool,
    verbose: bool,
    files: Vec<String>,
}

fn print_usage() {
    println!("destroy {} - securely overwrite files", VERSION);
    println!();
    println!("Usage: destroy [OPTIONS] FILE...");
    println!();
    println!("Options:");
    println!("  -h, --help       display this help message");
    println!("  -v, --version    display version");
    println!("  -n NUM           number of overwrite passes (default: 3)");
    println!("  -z               add a final pass of zeros");
    println!("  -u               remove (unlink) file after overwriting");
    println!("  -V               verbose - show progress");
}

fn parse_args() -> Options {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut opts = Options {
        passes: 3,
        zero_final: false,
        unlink: false,
        verbose: false,
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
                println!("destroy {}", VERSION);
                process::exit(0);
            }
            "-n" => {
                i += 1;
                if i < args.len() {
                    opts.passes = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("destroy: invalid number '{}'", args[i]);
                        process::exit(1);
                    });
                } else {
                    eprintln!("destroy: option requires an argument -- 'n'");
                    process::exit(1);
                }
            }
            "-z" => opts.zero_final = true,
            "-u" => opts.unlink = true,
            "-V" => opts.verbose = true,
            other => {
                opts.files.push(other.to_string());
            }
        }
        i += 1;
    }

    if opts.files.is_empty() {
        eprintln!("destroy: missing file operand");
        print_usage();
        process::exit(1);
    }

    opts
}

fn fill_buffer(buf: &mut [u8], pass: usize, _file_size: u64) {
    match pass % 4 {
        0 => {
            for b in buf.iter_mut() {
                *b = rand_byte();
            }
        }
        1 => {
            let pattern = (pass as u64).wrapping_mul(0x55) as u8;
            for b in buf.iter_mut() {
                *b = pattern;
            }
        }
        2 => {
            for (i, b) in buf.iter_mut().enumerate() {
                *b = (i % 256) as u8;
            }
        }
        _ => {
            for b in buf.iter_mut() {
                *b = 0xFF;
            }
        }
    }
}

fn rand_byte() -> u8 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let s = RandomState::new();
    let mut hasher = s.build_hasher();
    hasher.write_u64(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64);
    hasher.finish() as u8
}

fn destroy_file(path: &str, opts: &Options) {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("destroy: cannot stat '{}': {}", path, e);
            return;
        }
    };

    if meta.is_dir() {
        eprintln!("destroy: '{}': is a directory", path);
        return;
    }

    let size = meta.len();
    if size == 0 {
        if opts.unlink {
            fs::remove_file(path).ok();
        }
        return;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .open(path)
        .unwrap_or_else(|e| {
            eprintln!("destroy: cannot open '{}': {}", path, e);
            process::exit(1);
        });

    let block_size = 4096.min(size as usize);
    let mut buf = vec![0u8; block_size];

    for pass in 0..opts.passes {
        if opts.verbose {
            eprintln!("pass {}/{}: writing...", pass + 1, opts.passes);
        }

        file.seek(SeekFrom::Start(0)).unwrap();
        let mut written: u64 = 0;

        while written < size {
            let to_write = std::cmp::min(block_size as u64, size - written) as usize;
            fill_buffer(&mut buf[..to_write], pass, size);

            let mut offset = 0;
            while offset < to_write {
                match file.write(&buf[offset..to_write]) {
                    Ok(0) => break,
                    Ok(n) => offset += n,
                    Err(e) => {
                        eprintln!("destroy: write error on '{}': {}", path, e);
                        return;
                    }
                }
            }
            written += to_write as u64;
        }
        file.sync_all().ok();
    }

    if opts.zero_final {
        if opts.verbose {
            eprintln!("pass {}/{}: zeroing...", opts.passes + 1, opts.passes + 1);
        }
        file.seek(SeekFrom::Start(0)).unwrap();
        buf.fill(0);
        let mut written: u64 = 0;
        while written < size {
            let to_write = std::cmp::min(block_size as u64, size - written) as usize;
            let mut offset = 0;
            while offset < to_write {
                match file.write(&buf[offset..to_write]) {
                    Ok(0) => break,
                    Ok(n) => offset += n,
                    Err(e) => {
                        eprintln!("destroy: write error on '{}': {}", path, e);
                        return;
                    }
                }
            }
            written += to_write as u64;
        }
        file.sync_all().ok();
    }

    if opts.unlink {
        drop(file);
        fs::remove_file(path).unwrap_or_else(|e| {
            eprintln!("destroy: cannot remove '{}': {}", path, e);
        });
        if opts.verbose {
            eprintln!("removed '{}'", path);
        }
    }
}

fn main() {
    let opts = parse_args();

    for file in &opts.files {
        destroy_file(file, &opts);
    }
}

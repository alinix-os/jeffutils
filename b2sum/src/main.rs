use blake2::{Blake2b512, Digest};
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

fn usage() {
    eprintln!("Usage: b2sum [-l BITS] [-c] [FILE...]");
    eprintln!("  -l BITS    hash length in bits (default 512)");
    eprintln!("  -c         check mode: read hash file and verify");
    process::exit(1);
}

fn version() {
    println!("b2sum 0.1.0");
    process::exit(0);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        version();
    }

    let mut bits: usize = 512;
    let mut check_mode = false;
    let mut filenames: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-l" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("-l requires an argument");
                    process::exit(1);
                }
                bits = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("Invalid bit length");
                    process::exit(1);
                });
                if bits == 0 || bits > 512 || bits % 8 != 0 {
                    eprintln!("Bit length must be a multiple of 8 between 8 and 512");
                    process::exit(1);
                }
            }
            "-c" => check_mode = true,
            _ => filenames.push(args[i].clone()),
        }
        i += 1;
    }

    let bytes = bits / 8;

    if check_mode {
        check_hashes(&filenames, bytes);
    } else if filenames.is_empty() {
        let mut input = Vec::new();
        io::stdin().read_to_end(&mut input).unwrap();
        let hash = compute_hash(&input, bytes);
        println!("{}  -", hash);
    } else {
        for fname in &filenames {
            let mut file = File::open(fname).unwrap_or_else(|e| {
                eprintln!("{}: {}", fname, e);
                process::exit(1);
            });
            let mut input = Vec::new();
            file.read_to_end(&mut input).unwrap();
            let hash = compute_hash(&input, bytes);
            println!("{}  {}", hash, fname);
        }
    }
}

fn compute_hash(data: &[u8], bytes: usize) -> String {
    let mut hasher = Blake2b512::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(&result[..bytes])
}

fn check_hashes(filenames: &[String], bytes: usize) {
    if filenames.is_empty() {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).unwrap();
        verify_hash_lines(&input, bytes);
    } else {
        for fname in filenames {
            let mut file = File::open(fname).unwrap_or_else(|e| {
                eprintln!("{}: {}", fname, e);
                process::exit(1);
            });
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            verify_hash_lines(&content, bytes);
        }
    }
}

fn verify_hash_lines(content: &str, bytes: usize) {
    let mut all_ok = true;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Format: HASH  FILENAME
        let parts: Vec<&str> = line.splitn(2, "  ").collect();
        if parts.len() != 2 {
            eprintln!("Invalid line: {}", line);
            all_ok = false;
            continue;
        }
        let expected_hash = parts[0];
        let filename = parts[1];

        if filename == "-" {
            let mut input = Vec::new();
            io::stdin().read_to_end(&mut input).unwrap();
            let hash = compute_hash(&input, bytes);
            if hash == expected_hash {
                println!("{}: OK", filename);
            } else {
                eprintln!("{}: FAILED", filename);
                all_ok = false;
            }
        } else {
            match File::open(filename) {
                Ok(mut file) => {
                    let mut input = Vec::new();
                    file.read_to_end(&mut input).unwrap();
                    let hash = compute_hash(&input, bytes);
                    if hash == expected_hash {
                        println!("{}: OK", filename);
                    } else {
                        eprintln!("{}: FAILED", filename);
                        all_ok = false;
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", filename, e);
                    all_ok = false;
                }
            }
        }
    }
    if !all_ok {
        process::exit(1);
    }
}

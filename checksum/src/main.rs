use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

fn usage() {
    eprintln!("Usage: checksum [-a ALGO] [-l] [FILE...]");
    eprintln!("  -a ALGO   algorithm: crc32 (default), crc64");
    eprintln!("  -l        print length of checksum in bits");
    process::exit(1);
}

fn version() {
    println!("checksum 0.1.0");
    process::exit(0);
}

fn crc64(data: &[u8]) -> u64 {
    let mut crc: u64 = 0xffffffffffffffff;
    for &byte in data {
        crc ^= byte as u64;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0x95ac9329ac4bc9b5;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xffffffffffffffff
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("checksum", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        version();
    }

    let mut algo = "crc32".to_string();
    let mut print_length = false;
    let mut filenames: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("-a requires an argument");
                    process::exit(1);
                }
                algo = args[i].clone();
            }
            "-l" => print_length = true,
            _ => filenames.push(args[i].clone()),
        }
        i += 1;
    }

    if filenames.is_empty() {
        let mut input = Vec::new();
        io::stdin().read_to_end(&mut input).unwrap();
        let (checksum, bits) = compute_checksum(&input, &algo);
        if print_length {
            println!("{} {} {}", checksum, bits, "-");
        } else {
            println!("{} {} -", checksum, input.len());
        }
    } else {
        for fname in &filenames {
            let mut file = File::open(fname).unwrap_or_else(|e| {
                eprintln!("{}: {}", fname, e);
                process::exit(1);
            });
            let mut input = Vec::new();
            file.read_to_end(&mut input).unwrap();
            let (checksum, bits) = compute_checksum(&input, &algo);
            if print_length {
                println!("{} {} {}", checksum, bits, fname);
            } else {
                println!("{} {} {}", checksum, input.len(), fname);
            }
        }
    }
}

fn compute_checksum(data: &[u8], algo: &str) -> (String, usize) {
    match algo {
        "crc32" => {
            let checksum = crc32fast::hash(data);
            (format!("{:08x}", checksum), 32)
        }
        "crc64" => {
            let checksum = crc64(data);
            (format!("{:016x}", checksum), 64)
        }
        _ => {
            eprintln!("Unknown algorithm: {}", algo);
            process::exit(1);
        }
    }
}

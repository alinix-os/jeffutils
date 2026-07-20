use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

fn usage() {
    eprintln!("Usage: encode64 [-d] [-w COLS] [FILE...]");
    eprintln!("  -d        decode mode");
    eprintln!("  -w COLS   wrap at COLS characters (0 = no wrap, default 76)");
    process::exit(1);
}

fn version() {
    println!("encode64 0.1.0");
    process::exit(0);
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("encode64", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        version();
    }

    let mut decode = false;
    let mut wrap: usize = 76;
    let mut filenames: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" => decode = true,
            "-w" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("-w requires an argument");
                    process::exit(1);
                }
                wrap = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("Invalid wrap value");
                    process::exit(1);
                });
            }
            _ => filenames.push(args[i].clone()),
        }
        i += 1;
    }

    if filenames.is_empty() {
        let mut input = Vec::new();
        io::stdin().read_to_end(&mut input).unwrap();
        process_data(&input, decode, wrap);
    } else {
        for fname in &filenames {
            let mut file = File::open(fname).unwrap_or_else(|e| {
                eprintln!("{}: {}", fname, e);
                process::exit(1);
            });
            let mut input = Vec::new();
            file.read_to_end(&mut input).unwrap();
            process_data(&input, decode, wrap);
        }
    }
}

fn process_data(input: &[u8], decode: bool, wrap: usize) {
    if decode {
        let input_str = std::str::from_utf8(input).unwrap_or_else(|e| {
            eprintln!("Invalid UTF-8: {}", e);
            process::exit(1);
        });
        let cleaned: String = input_str.chars().filter(|c| !c.is_whitespace()).collect();
        let decoded = STANDARD.decode(&cleaned).unwrap_or_else(|e| {
            eprintln!("Decode error: {}", e);
            process::exit(1);
        });
        io::stdout().write_all(&decoded).unwrap();
    } else {
        let encoded = STANDARD.encode(input);
        if wrap == 0 {
            println!("{}", encoded);
        } else {
            for chunk in encoded.as_bytes().chunks(wrap) {
                println!("{}", std::str::from_utf8(chunk).unwrap());
            }
        }
    }
}

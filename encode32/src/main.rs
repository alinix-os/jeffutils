use base64::Engine;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

fn usage() {
    eprintln!("Usage: encode32 [-d] [FILE...]");
    eprintln!("  -d    decode mode");
    process::exit(1);
}

fn version() {
    println!("encode32 0.1.0");
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

    let mut decode = false;
    let mut filenames: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" => decode = true,
            _ => filenames.push(args[i].clone()),
        }
        i += 1;
    }

    if filenames.is_empty() {
        let mut input = Vec::new();
        io::stdin().read_to_end(&mut input).unwrap();
        process_data(&input, decode);
    } else {
        for fname in &filenames {
            let mut file = File::open(fname).unwrap_or_else(|e| {
                eprintln!("{}: {}", fname, e);
                process::exit(1);
            });
            let mut input = Vec::new();
            file.read_to_end(&mut input).unwrap();
            process_data(&input, decode);
        }
    }
}

fn process_data(input: &[u8], decode: bool) {
    if decode {
        let input_str = std::str::from_utf8(input).unwrap_or_else(|e| {
            eprintln!("Invalid UTF-8: {}", e);
            process::exit(1);
        });
        let cleaned: String = input_str
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        // Use the base32 engine - STANDARD engine does standard base32
        let engine = base64::engine::general_purpose::STANDARD;
        let decoded = engine.decode(&cleaned).unwrap_or_else(|e| {
            eprintln!("Decode error: {}", e);
            process::exit(1);
        });
        io::stdout().write_all(&decoded).unwrap();
    } else {
        // Base32 encode using the general purpose engine with base32 alphabet
        // base64 crate's STANDARD engine uses base64 alphabet, not base32
        // We need to use the base32 specific engine
        let encoded = base32_encode(input);
        println!("{}", encoded);
    }
}

fn base32_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut output = String::new();
    let mut buf: u32 = 0;
    let mut bits: i32 = 0;

    for &byte in input {
        buf = (buf << 8) | byte as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            output.push(ALPHABET[((buf >> bits) & 0x1f) as usize] as char);
        }
    }

    if bits > 0 {
        buf <<= 5 - bits;
        output.push(ALPHABET[((buf >> (5 - bits)) & 0x1f) as usize] as char);
    }

    // Add padding
    let padding = (8 - (output.len() % 8)) % 8;
    for _ in 0..padding {
        output.push('=');
    }

    output
}

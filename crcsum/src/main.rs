use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

fn usage() {
    eprintln!("Usage: crcsum [-r] [FILE...]");
    eprintln!("  -r    use reverse algorithm (SYSV sum replacement)");
    process::exit(1);
}

fn version() {
    println!("crcsum 0.1.0");
    process::exit(0);
}

fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x8005;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

fn crc16_reverse(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        version();
    }

    let mut reverse = false;
    let mut filenames: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-r" => reverse = true,
            _ => filenames.push(args[i].clone()),
        }
        i += 1;
    }

    if filenames.is_empty() {
        let mut input = Vec::new();
        io::stdin().read_to_end(&mut input).unwrap();
        let blocks = (input.len() + 511) / 512;
        let checksum = if reverse {
            crc16_reverse(&input)
        } else {
            crc16(&input)
        };
        println!("{} {}", checksum, blocks);
    } else {
        for fname in &filenames {
            let mut file = File::open(fname).unwrap_or_else(|e| {
                eprintln!("{}: {}", fname, e);
                process::exit(1);
            });
            let mut input = Vec::new();
            file.read_to_end(&mut input).unwrap();
            let blocks = (input.len() + 511) / 512;
            let checksum = if reverse {
                crc16_reverse(&input)
            } else {
                crc16(&input)
            };
            println!("{} {} {}", checksum, blocks, fname);
        }
    }
}

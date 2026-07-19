use std::env;
use std::io::{self, Read, BufReader};
use std::fs::File;
use std::process;

fn usage() {
    eprintln!("Usage: bytedump [OPTIONS] [FILE...]");
    eprintln!("  -t TYPE    output format: x(hex) o(octal) d(decimal) b(binary) c(char)");
    eprintln!("  -A RADIX   address radix: x o d n");
    eprintln!("  -j BYTES   skip BYTES input bytes");
    eprintln!("  -N BYTES   read only BYTES input bytes");
    process::exit(1);
}

fn version() {
    println!("bytedump 0.1.0");
    process::exit(0);
}

#[derive(Clone, Copy, PartialEq)]
enum OutputFormat {
    Hex,
    Octal,
    Decimal,
    Binary,
    Char,
}

#[derive(Clone, Copy, PartialEq)]
enum AddressRadix {
    Hex,
    Octal,
    Decimal,
    None,
}

fn format_byte(b: u8, fmt: OutputFormat) -> String {
    match fmt {
        OutputFormat::Hex => format!("{:02x}", b),
        OutputFormat::Octal => format!("{:03o}", b),
        OutputFormat::Decimal => format!("{:>3}", b),
        OutputFormat::Binary => format!("{:08b}", b),
        OutputFormat::Char => {
            if b >= 0x20 && b <= 0x7e {
                format!("  {}", b as char)
            } else {
                "  .".to_string()
            }
        }
    }
}

fn _bytes_per_group(fmt: OutputFormat) -> usize {
    match fmt {
        OutputFormat::Hex => 1,
        OutputFormat::Octal => 1,
        OutputFormat::Decimal => 1,
        OutputFormat::Binary => 1,
        OutputFormat::Char => 1,
    }
}

fn dump_chunk(chunk: &[u8], offset: u64, fmt: OutputFormat, addr_radix: AddressRadix) {
    if addr_radix != AddressRadix::None {
        let addr_str = match addr_radix {
            AddressRadix::Hex => format!("{:08x}", offset),
            AddressRadix::Octal => format!("{:011o}", offset),
            AddressRadix::Decimal => format!("{:010}", offset),
            AddressRadix::None => unreachable!(),
        };
        print!("{}  ", addr_str);
    }

    let bytes = format!(
        "{}",
        chunk
            .iter()
            .map(|b| format_byte(*b, fmt))
            .collect::<Vec<_>>()
            .join(" ")
    );

    if fmt == OutputFormat::Char {
        print!("{:<48}", bytes);
    } else {
        print!("{:<48}", bytes);
    }

    // Print ASCII representation
    let ascii: String = chunk
        .iter()
        .map(|&b| {
            if b >= 0x20 && b <= 0x7e {
                b as char
            } else {
                '.'
            }
        })
        .collect();
    println!("  |{}|", ascii);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        version();
    }

    let mut fmt = OutputFormat::Octal;
    let mut addr_radix = AddressRadix::Hex;
    let mut skip: u64 = 0;
    let mut max_bytes: Option<u64> = None;
    let mut filenames: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-t" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("-t requires an argument");
                    process::exit(1);
                }
                fmt = match args[i].as_str() {
                    "x" => OutputFormat::Hex,
                    "o" => OutputFormat::Octal,
                    "d" => OutputFormat::Decimal,
                    "b" => OutputFormat::Binary,
                    "c" => OutputFormat::Char,
                    _ => {
                        eprintln!("Invalid format: {}", args[i]);
                        process::exit(1);
                    }
                };
            }
            "-A" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("-A requires an argument");
                    process::exit(1);
                }
                addr_radix = match args[i].as_str() {
                    "x" => AddressRadix::Hex,
                    "o" => AddressRadix::Octal,
                    "d" => AddressRadix::Decimal,
                    "n" => AddressRadix::None,
                    _ => {
                        eprintln!("Invalid address radix: {}", args[i]);
                        process::exit(1);
                    }
                };
            }
            "-j" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("-j requires an argument");
                    process::exit(1);
                }
                skip = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("Invalid skip value");
                    process::exit(1);
                });
            }
            "-N" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("-N requires an argument");
                    process::exit(1);
                }
                max_bytes = Some(args[i].parse().unwrap_or_else(|_| {
                    eprintln!("Invalid byte count");
                    process::exit(1);
                }));
            }
            _ => {
                filenames.push(args[i].clone());
            }
        }
        i += 1;
    }

    if filenames.is_empty() {
        dump_reader(io::stdin(), addr_radix, fmt, skip, max_bytes, None);
    } else {
        for fname in &filenames {
            let file = File::open(fname).unwrap_or_else(|e| {
                eprintln!("{}: {}", fname, e);
                process::exit(1);
            });
            dump_reader(BufReader::new(file), addr_radix, fmt, skip, max_bytes, Some(fname));
        }
    }
}

fn dump_reader<R: Read>(mut reader: R, addr_radix: AddressRadix, fmt: OutputFormat, skip: u64, max_bytes: Option<u64>, _filename: Option<&str>) {
    let mut offset: u64 = 0;
    let mut total_read: u64 = 0;
    let mut buf = [0u8; 16];

    // Skip bytes
    if skip > 0 {
        let mut skipped = 0u64;
        let mut temp = [0u8; 1024];
        while skipped < skip {
            let to_read = std::cmp::min(1024, (skip - skipped) as usize);
            match reader.read(&mut temp[..to_read]) {
                Ok(0) => return,
                Ok(n) => skipped += n as u64,
                Err(_) => return,
            }
        }
        offset = skip;
    }

    loop {
        if let Some(max) = max_bytes {
            if total_read >= max {
                break;
            }
        }
        let to_read = std::cmp::min(16, max_bytes.map_or(16, |m| (m - total_read) as usize));
        let mut n = 0;
        while n < to_read {
            match reader.read(&mut buf[n..to_read]) {
                Ok(0) => break,
                Ok(bytes) => n += bytes,
                Err(_) => break,
            }
        }
        if n == 0 {
            break;
        }
        total_read += n as u64;
        dump_chunk(&buf[..n], offset, fmt, addr_radix);
        offset += n as u64;
    }
}

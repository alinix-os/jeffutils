use std::env;
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter, Seek, SeekFrom};
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Options {
    if_file: String,
    of_file: String,
    bs: usize,
    count: Option<u64>,
    skip: u64,
    seek: u64,
}

fn print_usage() {
    println!("blockcopy {} - copy files with block size control", VERSION);
    println!();
    println!("Usage: blockcopy [if=FILE] [of=FILE] [bs=SIZE] [count=N] [skip=N] [seek=N]");
    println!();
    println!("Options:");
    println!("  if=FILE       input file (default: stdin)");
    println!("  of=FILE       output file (default: stdout)");
    println!("  bs=SIZE       block size in bytes (default: 512)");
    println!("  count=N       copy only N blocks");
    println!("  skip=N        skip N blocks at start of input");
    println!("  seek=N        skip N blocks at start of output");
    println!("  -h, --help    display this help message");
    println!("  -v, --version display version");
}

fn parse_size(s: &str) -> Result<usize, String> {
    let s = s.trim();
    if let Some(val) = s.strip_suffix("K").or_else(|| s.strip_suffix("k")) {
        val.parse::<usize>().map(|v| v * 1024).map_err(|e| e.to_string())
    } else if let Some(val) = s.strip_suffix("M") {
        val.parse::<usize>().map(|v| v * 1_048_576).map_err(|e| e.to_string())
    } else if let Some(val) = s.strip_suffix("G") {
        val.parse::<usize>().map(|v| v * 1_073_741_824).map_err(|e| e.to_string())
    } else {
        s.parse::<usize>().map_err(|e| e.to_string())
    }
}

fn parse_args() -> Options {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut opts = Options {
        if_file: String::new(),
        of_file: String::new(),
        bs: 512,
        count: None,
        skip: 0,
        seek: 0,
    };

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-h" || arg == "--help" {
            print_usage();
            process::exit(0);
        } else if arg == "-v" || arg == "--version" {
            println!("blockcopy {}", VERSION);
            process::exit(0);
        } else if let Some(val) = arg.strip_prefix("if=") {
            opts.if_file = val.to_string();
        } else if let Some(val) = arg.strip_prefix("of=") {
            opts.of_file = val.to_string();
        } else if let Some(val) = arg.strip_prefix("bs=") {
            opts.bs = parse_size(val).unwrap_or_else(|e| {
                eprintln!("blockcopy: invalid block size '{}': {}", val, e);
                process::exit(1);
            });
        } else if let Some(val) = arg.strip_prefix("count=") {
            opts.count = val.parse().ok();
        } else if let Some(val) = arg.strip_prefix("skip=") {
            opts.skip = val.parse().unwrap_or(0);
        } else if let Some(val) = arg.strip_prefix("seek=") {
            opts.seek = val.parse().unwrap_or(0);
        } else {
            eprintln!("blockcopy: unknown option '{}'", arg);
            process::exit(1);
        }
        i += 1;
    }

    opts
}

fn do_copy<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    bs: usize,
    count: Option<u64>,
) -> (u64, u64, u64) {
    let mut buf = vec![0u8; bs];
    let mut blocks_read: u64 = 0;
    let mut blocks_written: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut blocks_remaining = count.unwrap_or(u64::MAX);

    loop {
        if blocks_remaining == 0 {
            break;
        }

        let mut bytes_read = 0;
        while bytes_read < bs {
            match reader.read(&mut buf[bytes_read..]) {
                Ok(0) => break,
                Ok(n) => bytes_read += n,
                Err(e) => {
                    eprintln!("blockcopy: read error: {}", e);
                    process::exit(1);
                }
            }
        }

        if bytes_read == 0 {
            break;
        }

        blocks_read += 1;

        let mut bytes_written = 0;
        while bytes_written < bytes_read {
            match writer.write(&buf[bytes_written..bytes_read]) {
                Ok(0) => {
                    eprintln!("blockcopy: write error: short write");
                    process::exit(1);
                }
                Ok(n) => bytes_written += n,
                Err(e) => {
                    eprintln!("blockcopy: write error: {}", e);
                    process::exit(1);
                }
            }
        }

        blocks_written += 1;
        total_bytes += bytes_read as u64;
        blocks_remaining -= 1;
    }

    (blocks_read, blocks_written, total_bytes)
}

fn main() {
    let opts = parse_args();

    if opts.if_file.is_empty() {
        let mut reader = std::io::stdin();
        if opts.of_file.is_empty() {
            let mut writer = std::io::stdout();
            let (ri, ro, bytes) = do_copy(&mut reader, &mut writer, opts.bs, opts.count);
            writer.flush().ok();
            eprintln!("{}+{} records in", ri, 0);
            eprintln!("{}+{} records out", ro, 0);
            eprintln!("{} bytes ({} B) copied", bytes, bytes);
        } else {
            let f = File::create(&opts.of_file).unwrap_or_else(|e| {
                eprintln!("blockcopy: cannot create '{}': {}", opts.of_file, e);
                process::exit(1);
            });
            let mut writer = BufWriter::with_capacity(opts.bs, f);
            if opts.seek > 0 {
                let seek_bytes = opts.seek * opts.bs as u64;
                writer.seek(SeekFrom::Start(seek_bytes)).unwrap_or_else(|e| {
                    eprintln!("blockcopy: cannot seek: {}", e);
                    process::exit(1);
                });
            }
            let (ri, ro, bytes) = do_copy(&mut reader, &mut writer, opts.bs, opts.count);
            writer.flush().ok();
            eprintln!("{}+{} records in", ri, 0);
            eprintln!("{}+{} records out", ro, 0);
            eprintln!("{} bytes ({} B) copied", bytes, bytes);
        }
    } else {
        let f = File::open(&opts.if_file).unwrap_or_else(|e| {
            eprintln!("blockcopy: cannot open '{}': {}", opts.if_file, e);
            process::exit(1);
        });
        let mut reader = BufReader::with_capacity(opts.bs, f);
        if opts.skip > 0 {
            let skip_bytes = opts.skip * opts.bs as u64;
            reader.seek(SeekFrom::Start(skip_bytes)).unwrap_or_else(|e| {
                eprintln!("blockcopy: cannot skip: {}", e);
                process::exit(1);
            });
        }
        if opts.of_file.is_empty() {
            let mut writer = std::io::stdout();
            let (ri, ro, bytes) = do_copy(&mut reader, &mut writer, opts.bs, opts.count);
            writer.flush().ok();
            eprintln!("{}+{} records in", ri, 0);
            eprintln!("{}+{} records out", ro, 0);
            eprintln!("{} bytes ({} B) copied", bytes, bytes);
        } else {
            let f = File::create(&opts.of_file).unwrap_or_else(|e| {
                eprintln!("blockcopy: cannot create '{}': {}", opts.of_file, e);
                process::exit(1);
            });
            let mut writer = BufWriter::with_capacity(opts.bs, f);
            if opts.seek > 0 {
                let seek_bytes = opts.seek * opts.bs as u64;
                writer.seek(SeekFrom::Start(seek_bytes)).unwrap_or_else(|e| {
                    eprintln!("blockcopy: cannot seek: {}", e);
                    process::exit(1);
                });
            }
            let (ri, ro, bytes) = do_copy(&mut reader, &mut writer, opts.bs, opts.count);
            writer.flush().ok();
            eprintln!("{}+{} records in", ri, 0);
            eprintln!("{}+{} records out", ro, 0);
            eprintln!("{} bytes ({} B) copied", bytes, bytes);
        }
    }
}

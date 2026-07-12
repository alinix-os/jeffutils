use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

fn print_usage() {
    eprintln!("Usage: read [FILE]...");
    eprintln!("Concatenate FILE(s) to standard output.");
    eprintln!("With no FILE, or when FILE is -, read standard input.");
}

fn copy_reader_to_stdout<R: Read>(mut reader: R) -> io::Result<()> {
    let mut buffer = [0; 8192];
    let mut stdout = io::stdout().lock();
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        stdout.write_all(&buffer[..bytes_read])?;
    }
    stdout.flush()?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // Check for help/version
    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "--version" {
            println!("read (JeffUtils) 1.0");
            return;
        }
    }

    if args.is_empty() {
        if let Err(e) = copy_reader_to_stdout(io::stdin().lock()) {
            eprintln!("read: error reading standard input: {}", e);
            std::process::exit(1);
        }
    } else {
        for arg in &args {
            if arg == "-" {
                if let Err(e) = copy_reader_to_stdout(io::stdin().lock()) {
                    eprintln!("read: error reading standard input: {}", e);
                    std::process::exit(1);
                }
            } else {
                let path = Path::new(arg);
                match File::open(path) {
                    Ok(file) => {
                        if let Err(e) = copy_reader_to_stdout(file) {
                            eprintln!("read: error reading '{}': {}", arg, e);
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("read: '{}': {}", arg, e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

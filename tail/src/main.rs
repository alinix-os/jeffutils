use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

fn print_usage() {
    eprintln!("Usage: tail [OPTION]... [FILE]...");
    eprintln!("Print the last 10 lines of each FILE to standard output.");
    eprintln!("With more than one FILE, precede each with a header giving the file name.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -n, --lines=NUM      output the last NUM lines (default 10)");
    eprintln!("  -h, --help           display this help and exit");
    eprintln!("      --version        output version information and exit");
}

fn tail_reader<R: BufRead>(mut reader: R, num_lines: usize) -> io::Result<()> {
    if num_lines == 0 {
        return Ok(());
    }
    let mut buffer = VecDeque::with_capacity(num_lines);
    let mut line = String::new();
    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }
        if buffer.len() == num_lines {
            buffer.pop_front();
        }
        buffer.push_back(line.clone());
    }

    let mut stdout = io::stdout().lock();
    for l in buffer {
        stdout.write_all(l.as_bytes())?;
    }
    stdout.flush()?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "--version" {
            println!("tail (JeffUtils) 1.0");
            return;
        }
    }

    let mut num_lines = 10;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        if args[i] == "-n" || args[i] == "--lines" {
            i += 1;
            if i < args.len() {
                if let Ok(n) = args[i].parse::<usize>() {
                    num_lines = n;
                } else {
                    eprintln!("tail: invalid number of lines: '{}'", args[i]);
                    std::process::exit(1);
                }
            }
        } else if args[i].starts_with("-n") {
            let val = &args[i][2..];
            if let Ok(n) = val.parse::<usize>() {
                num_lines = n;
            } else {
                eprintln!("tail: invalid number of lines: '{}'", val);
                std::process::exit(1);
            }
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }

    if files.is_empty() {
        if let Err(e) = tail_reader(io::stdin().lock(), num_lines) {
            eprintln!("tail: error reading stdin: {}", e);
            std::process::exit(1);
        }
    } else {
        let print_headers = files.len() > 1;
        for (idx, filename) in files.iter().enumerate() {
            if print_headers {
                if idx > 0 {
                    println!();
                }
                println!("==> {} <==", filename);
            }

            if filename == "-" {
                if let Err(e) = tail_reader(io::stdin().lock(), num_lines) {
                    eprintln!("tail: error reading stdin: {}", e);
                    std::process::exit(1);
                }
            } else {
                match File::open(filename) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = tail_reader(reader, num_lines) {
                            eprintln!("tail: error reading '{}': {}", filename, e);
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("tail: cannot open '{}' for reading: {}", filename, e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

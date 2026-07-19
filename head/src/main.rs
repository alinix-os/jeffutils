use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

fn print_usage() {
    eprintln!("Usage: head [OPTION]... [FILE]... ");
    eprintln!("Print the first 10 lines of each FILE to standard output.");
    eprintln!("With more than one FILE, precede each with a header giving the file name.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -n, --lines=NUM      print the first NUM lines (default 10)");
    eprintln!("  -h, --help           display this help and exit");
    eprintln!("      --version        output version information and exit");
}

fn head_reader<R: BufRead>(mut reader: R, num_lines: usize) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    let mut count = 0;
    let mut line = String::new();
    while count < num_lines {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }
        match stdout.write_all(line.as_bytes()) {
            Ok(()) => {}
            Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => std::process::exit(0),
            Err(e) => return Err(e),
        }
        count += 1;
    }
    let _ = stdout.flush();
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
            println!("head (JeffUtils) 1.0");
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
                    eprintln!("head: invalid number of lines: '{}'", args[i]);
                    std::process::exit(1);
                }
            } else {
                eprintln!("head: option requires an argument -- 'lines'");
                std::process::exit(1);
            }
        } else if let Some(val) = args[i].strip_prefix("--lines=") {
            if let Ok(n) = val.parse::<usize>() {
                num_lines = n;
            } else {
                eprintln!("head: invalid number of lines: '{}'", val);
                std::process::exit(1);
            }
        } else if args[i].starts_with("-n") {
            let val = &args[i][2..];
            if let Ok(n) = val.parse::<usize>() {
                num_lines = n;
            } else {
                eprintln!("head: invalid number of lines: '{}'", val);
                std::process::exit(1);
            }
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }

    let mut exit_code = 0;

    if files.is_empty() {
        if let Err(e) = head_reader(io::stdin().lock(), num_lines) {
            eprintln!("head: error reading stdin: {}", e);
            exit_code = 1;
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
                if let Err(e) = head_reader(io::stdin().lock(), num_lines) {
                    eprintln!("head: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(filename) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = head_reader(reader, num_lines) {
                            eprintln!("head: error reading '{}': {}", filename, e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("head: cannot open '{}' for reading: {}", filename, e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    std::process::exit(exit_code);
}

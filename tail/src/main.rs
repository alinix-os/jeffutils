use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::thread;
use std::time::Duration;

fn print_usage() {
    eprintln!("Usage: tail [OPTION]... [FILE]...");
    eprintln!("Print the last 10 lines of each FILE to standard output.");
    eprintln!("With more than one FILE, precede each with a header giving the file name.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -f, --follow          output appended data as the file grows");
    eprintln!("  -n, --lines=NUM      output the last NUM lines (default 10)");
    eprintln!("  -h, --help           display this help and exit");
    eprintln!("      --version        output version information and exit");
}

fn write_line(line: &str) {
    let mut stdout = io::stdout();
    match stdout.write_all(line.as_bytes()) {
        Ok(()) => {}
        Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => std::process::exit(0),
        Err(_) => {}
    }
    let _ = stdout.flush();
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
        match stdout.write_all(l.as_bytes()) {
            Ok(()) => {}
            Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => std::process::exit(0),
            Err(e) => return Err(e),
        }
    }
    let _ = stdout.flush();
    Ok(())
}

fn tail_follow_stdin(num_lines: usize) {
    let mut reader = BufReader::new(io::stdin().lock());
    let mut buffer = VecDeque::with_capacity(num_lines);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if buffer.len() == num_lines {
                    buffer.pop_front();
                }
                buffer.push_back(line.clone());
            }
            Err(_) => break,
        }
    }

    for l in &buffer {
        write_line(l);
    }

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                thread::sleep(Duration::from_secs(1));
            }
            Ok(_) => {
                write_line(&line);
            }
            Err(_) => {
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn tail_follow_file(filename: &str) {
    loop {
        match File::open(filename) {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                use std::io::Seek;
                let _ = reader.seek(io::SeekFrom::End(0));

                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) => {
                            thread::sleep(Duration::from_secs(1));
                        }
                        Ok(_) => {
                            write_line(&line);
                        }
                        Err(_) => {
                            thread::sleep(Duration::from_secs(1));
                        }
                    }
                }
            }
            Err(_) => {
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
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
    let mut follow = false;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        if args[i] == "-f" || args[i] == "--follow" {
            follow = true;
        } else if args[i] == "-n" || args[i] == "--lines" {
            i += 1;
            if i < args.len() {
                if let Ok(n) = args[i].parse::<usize>() {
                    num_lines = n;
                } else {
                    eprintln!("tail: invalid number of lines: '{}'", args[i]);
                    std::process::exit(1);
                }
            } else {
                eprintln!("tail: option requires an argument -- 'lines'");
                std::process::exit(1);
            }
        } else if let Some(val) = args[i].strip_prefix("--lines=") {
            if let Ok(n) = val.parse::<usize>() {
                num_lines = n;
            } else {
                eprintln!("tail: invalid number of lines: '{}'", val);
                std::process::exit(1);
            }
        } else if args[i].starts_with("-n") {
            let val = &args[i][2..];
            if let Ok(n) = val.parse::<usize>() {
                num_lines = n;
            } else {
                eprintln!("tail: invalid number of lines: '{}'", val);
                std::process::exit(1);
            }
        } else if args[i].starts_with('-') && args[i].len() > 1 && args[i][1..].chars().all(|c| c.is_ascii_digit()) {
            let val = &args[i][1..];
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

    let mut exit_code = 0;

    if files.is_empty() {
        if follow {
            tail_follow_stdin(num_lines);
        } else {
            if let Err(e) = tail_reader(io::stdin().lock(), num_lines) {
                eprintln!("tail: error reading stdin: {}", e);
                exit_code = 1;
            }
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
                if follow {
                    tail_follow_stdin(num_lines);
                } else {
                    if let Err(e) = tail_reader(io::stdin().lock(), num_lines) {
                        eprintln!("tail: error reading stdin: {}", e);
                        exit_code = 1;
                    }
                }
            } else {
                match File::open(filename) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = tail_reader(reader, num_lines) {
                            eprintln!("tail: error reading '{}': {}", filename, e);
                            exit_code = 1;
                        }
                        if follow {
                            tail_follow_file(filename);
                        }
                    }
                    Err(e) => {
                        eprintln!("tail: cannot open '{}' for reading: {}", filename, e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    std::process::exit(exit_code);
}

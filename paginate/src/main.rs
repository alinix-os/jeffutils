use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    eprintln!("Usage: paginate [OPTIONS] [FILE...]");
    eprintln!("Paginate text for printing.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -l LENGTH    Page length in lines (default: 66)");
    eprintln!("  -h HEADER    Header text");
    eprintln!("  -w WIDTH     Page width in characters (default: 72)");
    eprintln!("  --help       Show this help message");
    eprintln!("  --version    Show version");
}

fn paginate<R: BufRead>(
    reader: R,
    page_length: usize,
    header: &str,
    page_width: usize,
    page_num: &mut usize,
) {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut lines_on_page = 0;
    let header_lines = if header.is_empty() { 0 } else { 3 };

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break,
        };

        if lines_on_page == 0 {
            *page_num += 1;
            if !header.is_empty() {
                writeln!(out, "{header:-<width$}", width = page_width).ok();
                writeln!(out, "Page {page_num}").ok();
                writeln!(out).ok();
                lines_on_page = header_lines;
            }
        }

        let truncated: String = line.chars().take(page_width).collect();
        writeln!(out, "{truncated}").ok();
        lines_on_page += 1;

        if lines_on_page >= page_length {
            writeln!(out, "\x0c").ok();
            lines_on_page = 0;
        }
    }

    if lines_on_page > 0 {
        writeln!(out, "\x0c").ok();
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut page_length: usize = 66;
    let mut header = String::new();
    let mut page_width: usize = 72;
    let mut files = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--help" => {
                print_help();
                return;
            }
            "--version" => {
                println!("paginate {VERSION}");
                return;
            }
            "-l" => {
                i += 1;
                if i < args.len() {
                    page_length = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("paginate: invalid page length '{}'", args[i]);
                        std::process::exit(1);
                    });
                } else {
                    eprintln!("paginate: -l requires an argument");
                    std::process::exit(1);
                }
            }
            "-h" => {
                i += 1;
                if i < args.len() {
                    header = args[i].clone();
                } else {
                    eprintln!("paginate: -h requires an argument");
                    std::process::exit(1);
                }
            }
            "-w" => {
                i += 1;
                if i < args.len() {
                    page_width = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("paginate: invalid page width '{}'", args[i]);
                        std::process::exit(1);
                    });
                } else {
                    eprintln!("paginate: -w requires an argument");
                    std::process::exit(1);
                }
            }
            "--" => {
                i += 1;
                while i < args.len() {
                    files.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            _ if args[i].starts_with('-') => {
                eprintln!("paginate: unknown option '{}'", args[i]);
                std::process::exit(1);
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    let mut page_num = 0;

    if files.is_empty() {
        let stdin = io::stdin();
        paginate(stdin.lock(), page_length, &header, page_width, &mut page_num);
    } else {
        for path in &files {
            let file = File::open(path).unwrap_or_else(|e| {
                eprintln!("paginate: cannot read '{path}': {e}");
                std::process::exit(1);
            });
            paginate(
                BufReader::new(file),
                page_length,
                &header,
                page_width,
                &mut page_num,
            );
        }
    }
}

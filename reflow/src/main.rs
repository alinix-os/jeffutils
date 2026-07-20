use std::env;
use std::io::{self, BufRead, Write};

fn print_usage() {
    eprintln!("Usage: reflow [OPTION]... [FILE]...");
    eprintln!("Reformat paragraphs to target width.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -w WIDTH   target width (default 75)");
    eprintln!("  -s         only split long lines, don't join short ones");
    eprintln!("  -t         use tabs for indentation");
    eprintln!("  -h, --help       display this help and exit");
    eprintln!("  -v, --version    display version and exit");
}

fn reflow_paragraph(lines: &[&str], width: usize, split_only: bool, use_tabs: bool) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut indent = String::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if i == 0 {
            // Detect indentation from first line
            let leading = line.len() - line.trim_start().len();
            let lead_chars: String = line.chars().take(leading).collect();
            if use_tabs {
                let tabs = (leading + 3) / 4;
                indent = "\t".repeat(tabs);
            } else {
                indent = lead_chars;
            }
        }
        for word in trimmed.split_whitespace() {
            words.push(word.to_string());
        }
    }

    if words.is_empty() {
        return vec![String::new()];
    }

    if split_only {
        // Don't join, just return original lines
        return lines.iter().map(|l| l.to_string()).collect();
    }

    let mut result = Vec::new();
    let mut current_line = String::new();
    let _indent_len = indent.len();

    for word in &words {
        if current_line.is_empty() {
            current_line = format!("{}{}", indent, word);
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            result.push(current_line);
            current_line = format!("{}{}", indent, word);
        }
    }

    if !current_line.is_empty() {
        result.push(current_line);
    }

    if result.is_empty() {
        result.push(String::new());
    }

    result
}

fn reflow_reader(reader: &mut dyn BufRead, width: usize, split_only: bool, use_tabs: bool) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut paragraph_lines: Vec<String> = Vec::new();

    let mut buffer = String::new();
    loop {
        buffer.clear();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                // Flush last paragraph
                if !paragraph_lines.is_empty() {
                    let refs: Vec<&str> = paragraph_lines.iter().map(|s| s.as_str()).collect();
                    let reflowed = reflow_paragraph(&refs, width, split_only, use_tabs);
                    for line in &reflowed {
                        writeln!(out, "{}", line)?;
                    }
                    paragraph_lines.clear();
                }
                break;
            }
            Ok(_) => {
                let trimmed = buffer.trim_end_matches('\n').trim_end_matches('\r');

                if trimmed.trim().is_empty() {
                    // Blank line: flush paragraph
                    if !paragraph_lines.is_empty() {
                        let refs: Vec<&str> = paragraph_lines.iter().map(|s| s.as_str()).collect();
                        let reflowed = reflow_paragraph(&refs, width, split_only, use_tabs);
                        for line in &reflowed {
                            writeln!(out, "{}", line)?;
                        }
                        paragraph_lines.clear();
                    }
                    writeln!(out)?;
                } else {
                    paragraph_lines.push(trimmed.to_string());
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("reflow", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("reflow (JeffUtils) 1.0");
        return;
    }

    let mut width = 75;
    let mut split_only = false;
    let mut use_tabs = false;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" => split_only = true,
            "-t" => use_tabs = true,
            "-w" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("reflow: option '-w' requires an argument");
                    std::process::exit(1);
                }
                width = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("reflow: invalid width '{}'", args[i]);
                    std::process::exit(1);
                });
            }
            "--version" => {}
            "--help" => {}
            other if other.starts_with('-') && other.len() > 1 => {
                for ch in other[1..].chars() {
                    match ch {
                        's' => split_only = true,
                        't' => use_tabs = true,
                        'w' => {
                            eprintln!("reflow: option '-w' requires an argument");
                            std::process::exit(1);
                        }
                        _ => {
                            eprintln!("reflow: unknown option '-{}'", ch);
                            std::process::exit(1);
                        }
                    }
                }
            }
            _ => {
                files.push(args[i].clone());
            }
        }
        i += 1;
    }

    if files.is_empty() {
        let mut stdin = io::BufReader::new(io::stdin());
        if let Err(e) = reflow_reader(&mut stdin, width, split_only, use_tabs) {
            eprintln!("reflow: {}", e);
            std::process::exit(1);
        }
    } else {
        for file in &files {
            match std::fs::File::open(file) {
                Ok(f) => {
                    let mut reader = io::BufReader::new(f);
                    if let Err(e) = reflow_reader(&mut reader, width, split_only, use_tabs) {
                        eprintln!("reflow: {}: {}", file, e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("reflow: {}: {}", file, e);
                    std::process::exit(1);
                }
            }
        }
    }
}

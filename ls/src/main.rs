use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

const RESET: &str = "\x1B[0m";
const BLUE: &str = "\x1B[1;34m";
const GREEN: &str = "\x1B[1;32m";
const RED: &str = "\x1B[1;31m";
const YELLOW: &str = "\x1B[1;33m";
const GRAY: &str = "\x1B[2;37m";
const WHITE: &str = "\x1B[0;37m";

fn print_usage() {
    eprintln!("Usage: {} [caminho] [-l] [-a] [-h]", std::env::args().nth(0).unwrap_or_else(|| "ls".into()));
}

fn file_color(mode: u32) -> &'static str {
    let file_type = mode & 0o170000;
    match file_type {
        0o040000 => BLUE,
        0o100000 => {
            if mode & 0o111 != 0 { GREEN } else { WHITE }
        }
        _ => WHITE,
    }
}

fn format_mode(mode: u32) -> String {
    let file_type = match mode & 0o170000 {
        0o140000 => "s", 0o120000 => "l", 0o100000 => "-",
        0o060000 => "b", 0o040000 => "d", 0o020000 => "c",
        0o010000 => "p", _ => "?",
    };
    let r  = if mode & 0o400 != 0 { "r" } else { "-" };
    let w  = if mode & 0o200 != 0 { "w" } else { "-" };
    let x  = if mode & 0o100 != 0 { "x" } else { "-" };
    let r2 = if mode & 0o040 != 0 { "r" } else { "-" };
    let w2 = if mode & 0o020 != 0 { "w" } else { "-" };
    let x2 = if mode & 0o010 != 0 { "x" } else { "-" };
    let r3 = if mode & 0o004 != 0 { "r" } else { "-" };
    let w3 = if mode & 0o002 != 0 { "w" } else { "-" };
    let x3 = if mode & 0o001 != 0 { "x" } else { "-" };
    format!("{}{}{}{}{}{}{}{}{}{}", file_type, r, w, x, r2, w2, x2, r3, w3, x3)
}

fn format_size(size: u64, human: bool) -> String {
    if human {
        const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
        let mut s = size as f64;
        let mut i = 0;
        while s >= 1024.0 && i < UNITS.len() - 1 { s /= 1024.0; i += 1; }
        format!("{:>4.1}{}", s, UNITS[i])
    } else {
        format!("{:>8}", size)
    }
}

fn get_entry_color(entry: &fs::DirEntry, all: bool) -> &'static str {
    let name = entry.file_name();
    let name_str = name.to_string_lossy();
    if all && name_str.starts_with('.') {
        return GRAY;
    }
    if name_str.ends_with(".lock") {
        return YELLOW;
    }
    let metadata = match entry.metadata() {
        Ok(m) => m,
        Err(_) => return RED,
    };
    file_color(metadata.permissions().mode())
}

fn list_directory(path: &Path, long: bool, all: bool, human: bool) {
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => { eprintln!("Error reading {}: {}", path.display(), e); return; }
    };

    let mut files: Vec<_> = entries.flatten().collect();
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    if long {
        for entry in &files {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if !all && name_str.starts_with('.') { continue; }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let color = get_entry_color(entry, all);
            let perms = format_mode(metadata.permissions().mode());
            let size_str = format_size(metadata.len(), human);
            println!("{} {:>3} {:>5} {:>5} {:>8} {}{}{}", perms, metadata.nlink(), metadata.uid(), metadata.gid(), size_str, color, name_str, RESET);
        }
    } else {
        let mut colored: Vec<(String, &str)> = Vec::new();
        for entry in &files {
            let name_str = entry.file_name().to_string_lossy().to_string();
            if !all && name_str.starts_with('.') { continue; }
            colored.push((name_str, get_entry_color(entry, all)));
        }

        let max_len = colored.iter().map(|(s, _)| s.len()).max().unwrap_or(0) + 2;
        let cols = 4;
        for (i, (name, color)) in colored.iter().enumerate() {
            print!("{}{}{}{:width$}", color, name, RESET, "", width = max_len.saturating_sub(name.len()));
            if (i + 1) % cols == 0 { println!(); }
        }
        if !colored.is_empty() { println!(); }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut path = ".".to_string();
    let mut long = false;
    let mut all = false;
    let mut human = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Lists directory contents.");
                println!("  -l       Long format");
                println!("  -a       Show hidden files");
                println!("  -h       Human-readable sizes");
                println!("  --help, -h  Show this help message");
                println!("  --version   Show version information");
                return;
            }
            "--version" => {
                println!("ls version 0.1.0");
                return;
            }
            "-l" => long = true,
            "-a" => all = true,
            "-la" | "-al" => { long = true; all = true; }
            "-lh" | "-hl" => { long = true; human = true; }
            "-lah" | "-lha" | "-alh" | "-ahl" | "-hal" | "-hla" => { long = true; all = true; human = true; }
            _ => path = args[i].clone(),
        }
        i += 1;
    }

    let path = Path::new(&path);
    if !path.exists() {
        eprintln!("Error: path '{}' not found", path.display());
        std::process::exit(1);
    }

    if path.is_dir() {
        list_directory(path, long, all, human);
    } else {
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => { eprintln!("Error: {}", e); std::process::exit(1); }
        };
        let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        let color = file_color(metadata.permissions().mode());
        if long {
            let perms = format_mode(metadata.permissions().mode());
            let size_str = format_size(metadata.len(), human);
            println!("{} {:>3} {:>5} {:>5} {:>8} {}{}{}", perms, metadata.nlink(), metadata.uid(), metadata.gid(), size_str, color, name, RESET);
        } else {
            println!("{}{}{}", color, name, RESET);
        }
    }
}

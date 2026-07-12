use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::io::IsTerminal;

const RESET: &str = "\x1B[0m";
// Normal colors
const BLUE: &str = "\x1B[1;34m";
const GREEN: &str = "\x1B[1;32m";
const RED: &str = "\x1B[1;31m";
const YELLOW: &str = "\x1B[1;33m";
const GRAY: &str = "\x1B[90m";
const WHITE: &str = "\x1B[0;37m";
const CYAN: &str = "\x1B[1;36m";
const MAGENTA: &str = "\x1B[1;35m";

// Grayish/dimmed colors for hidden files
const GRAY_BLUE: &str = "\x1B[38;5;110m";    // Soft steel blue
const GRAY_GREEN: &str = "\x1B[38;5;108m";   // Soft sage green
const GRAY_CYAN: &str = "\x1B[38;5;152m";    // Soft grayish cian
const GRAY_MAGENTA: &str = "\x1B[38;5;139m"; // Soft grayish magenta
const GRAY_YELLOW: &str = "\x1B[38;5;143m";  // Soft grayish yellow

fn print_usage() {
    eprintln!("Usage: {} [caminho] [-l] [-a] [-h] [--color=WHEN]", std::env::args().nth(0).unwrap_or_else(|| "ls".into()));
}

fn get_terminal_width() -> usize {
    #[cfg(unix)]
    {
        use std::mem::zeroed;
        unsafe {
            let mut ws: libc::winsize = zeroed();
            if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
                return ws.ws_col as usize;
            }
        }
    }
    if let Ok(cols_str) = std::env::var("COLUMNS") {
        if let Ok(cols) = cols_str.parse::<usize>() {
            return cols;
        }
    }
    80
}

fn file_color(mode: u32, is_hidden: bool) -> &'static str {
    let file_type = mode & 0o170000;
    if is_hidden {
        match file_type {
            0o040000 => GRAY_BLUE,    // hidden directory
            0o120000 => GRAY_CYAN,    // hidden symlink
            0o140000 => GRAY_MAGENTA, // hidden socket
            0o010000 => GRAY_MAGENTA, // hidden FIFO/pipe
            0o060000 => GRAY_YELLOW,  // hidden block device
            0o020000 => GRAY_YELLOW,  // hidden character device
            0o100000 => {
                if mode & 0o111 != 0 { GRAY_GREEN } else { GRAY }
            }
            _ => GRAY,
        }
    } else {
        match file_type {
            0o040000 => BLUE,    // directory
            0o120000 => CYAN,    // symlink
            0o140000 => MAGENTA, // socket
            0o010000 => MAGENTA, // FIFO/pipe
            0o060000 => YELLOW,  // block device
            0o020000 => YELLOW,  // character device
            0o100000 => {
                if mode & 0o111 != 0 { GREEN } else { WHITE }
            }
            _ => WHITE,
        }
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

fn get_entry_color(entry: &fs::DirEntry, _all: bool, use_color: bool) -> &'static str {
    if !use_color {
        return "";
    }
    let name = entry.file_name();
    let name_str = name.to_string_lossy();
    let is_hidden = name_str.starts_with('.');
    if name_str.ends_with(".lock") {
        return if is_hidden { GRAY_YELLOW } else { YELLOW };
    }
    let metadata = match entry.metadata() {
        Ok(m) => m,
        Err(_) => return RED,
    };
    file_color(metadata.permissions().mode(), is_hidden)
}

fn list_directory(path: &Path, long: bool, all: bool, human: bool, use_color: bool) {
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => { eprintln!("Error reading {}: {}", path.display(), e); return; }
    };

    let mut files: Vec<_> = entries.flatten().collect();
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let mut visible_colored: Vec<(String, &str)> = Vec::new();
    let mut hidden_colored: Vec<(String, &str)> = Vec::new();

    for entry in &files {
        let name_str = entry.file_name().to_string_lossy().to_string();
        if name_str.starts_with('.') {
            if all {
                hidden_colored.push((name_str, get_entry_color(entry, all, use_color)));
            }
        } else {
            visible_colored.push((name_str, get_entry_color(entry, all, use_color)));
        }
    }

    if all {
        let folder_color = if use_color {
            if let Ok(meta) = fs::metadata(path) {
                file_color(meta.permissions().mode(), true)
            } else {
                GRAY_BLUE
            }
        } else {
            ""
        };
        hidden_colored.insert(0, ("..".to_string(), folder_color));
        hidden_colored.insert(0, (".".to_string(), folder_color));
    }

    let mut colored = visible_colored;
    colored.extend(hidden_colored);

    if long {
        for (name, color) in &colored {
            let metadata_res = if name == "." {
                fs::metadata(path)
            } else if name == ".." {
                path.parent()
                    .or_else(|| Some(Path::new("..")))
                    .map(fs::metadata)
                    .unwrap_or_else(|| fs::metadata(path))
            } else {
                fs::metadata(path.join(name))
            };

            match metadata_res {
                Ok(metadata) => {
                    let perms = format_mode(metadata.permissions().mode());
                    let size_str = format_size(metadata.len(), human);
                    let reset = if use_color { RESET } else { "" };
                    println!("{} {:>3} {:>5} {:>5} {:>8} {}{}{}", perms, metadata.nlink(), metadata.uid(), metadata.gid(), size_str, color, name, reset);
                }
                Err(_) => {
                    println!("?????????   ?     ?     ?        ? {}{}{}", color, name, if use_color { RESET } else { "" });
                }
            }
        }
    } else {
        if colored.is_empty() {
            return;
        }

        let term_width = get_terminal_width();
        let max_name_len = colored.iter().map(|(s, _)| s.len()).max().unwrap_or(0);
        let col_width = max_name_len + 2;

        let num_cols = (term_width / col_width).max(1);
        let num_rows = (colored.len() + num_cols - 1) / num_cols;

        let reset = if use_color { RESET } else { "" };

        for r in 0..num_rows {
            for c in 0..num_cols {
                let idx = c * num_rows + r;
                if idx < colored.len() {
                    let (name, color) = &colored[idx];
                    if c == num_cols - 1 || idx + num_rows >= colored.len() {
                        print!("{}{}{}", color, name, reset);
                    } else {
                        print!("{}{}{}{:width$}", color, name, reset, "", width = col_width - name.len());
                    }
                }
            }
            println!();
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut path = ".".to_string();
    let mut long = false;
    let mut all = false;
    let mut human = false;
    let mut color_opt = "auto".to_string();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with("--color=") {
            color_opt = arg.trim_start_matches("--color=").to_string();
        } else if arg == "--color" {
            color_opt = "always".to_string();
        } else {
            match arg.as_str() {
                "--help" | "-h" => {
                    print_usage();
                    println!("Lists directory contents.");
                    println!("  -l       Long format");
                    println!("  -a       Show hidden files");
                    println!("  -h       Human-readable sizes");
                    println!("  --color[=WHEN]  Control whether color is used: always, never, auto");
                    println!("  --help, -h  Show this help message");
                    println!("  --version   Show version information");
                    return;
                }
                "--version" => {
                    println!("ls version 0.4.0");
                    return;
                }
                "-l" => long = true,
                "-a" => all = true,
                "-la" | "-al" => { long = true; all = true; }
                "-lh" | "-hl" => { long = true; human = true; }
                "-lah" | "-lha" | "-alh" | "-ahl" | "-hal" | "-hla" => { long = true; all = true; human = true; }
                _ => {
                    if !arg.starts_with("-") {
                        path = arg.clone();
                    }
                }
            }
        }
        i += 1;
    }

    let use_color = match color_opt.as_str() {
        "always" => true,
        "never" => false,
        _ => std::io::stdout().is_terminal(),
    };

    let path = Path::new(&path);
    if !path.exists() {
        eprintln!("Error: path '{}' not found", path.display());
        std::process::exit(1);
    }

    if path.is_dir() {
        list_directory(path, long, all, human, use_color);
    } else {
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => { eprintln!("Error: {}", e); std::process::exit(1); }
        };
        let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        let color = if use_color { file_color(metadata.permissions().mode(), name.starts_with('.')) } else { "" };
        let reset = if use_color { RESET } else { "" };
        if long {
            let perms = format_mode(metadata.permissions().mode());
            let size_str = format_size(metadata.len(), human);
            println!("{} {:>3} {:>5} {:>5} {:>8} {}{}{}", perms, metadata.nlink(), metadata.uid(), metadata.gid(), size_str, color, name, reset);
        } else {
            println!("{}{}{}", color, name, reset);
        }
    }
}

use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use std::env;

struct CleanStats {
    files_deleted: u64,
    dirs_deleted: u64,
    bytes_freed: u64,
    errors: u64,
}

impl CleanStats {
    fn new() -> Self {
        CleanStats {
            files_deleted: 0,
            dirs_deleted: 0,
            bytes_freed: 0,
            errors: 0,
        }
    }

    fn merge(&mut self, other: &CleanStats) {
        self.files_deleted += other.files_deleted;
        self.dirs_deleted += other.dirs_deleted;
        self.bytes_freed += other.bytes_freed;
        self.errors += other.errors;
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn clean_directory(path: &Path, dry_run: bool, verbose: bool) -> CleanStats {
    let mut stats = CleanStats::new();
    
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => {
            // Silently skip unreadable directories
            return stats;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path_buf = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.is_dir() {
            // Recursively clean subdirectories first
            let sub_stats = clean_directory(&path_buf, dry_run, verbose);
            stats.merge(&sub_stats);

            // Attempt to remove the directory itself if it's empty
            if !dry_run {
                match fs::remove_dir(&path_buf) {
                    Ok(_) => {
                        stats.dirs_deleted += 1;
                        if verbose {
                            println!("Removed directory: {}", path_buf.display());
                        }
                    }
                    Err(_) => {
                        // Directory might not be empty or locked; skip
                        stats.errors += 1;
                    }
                }
            } else {
                stats.dirs_deleted += 1;
            }
        } else {
            let file_size = metadata.len();
            if !dry_run {
                match fs::remove_file(&path_buf) {
                    Ok(_) => {
                        stats.files_deleted += 1;
                        stats.bytes_freed += file_size;
                        if verbose {
                            println!("Removed file ({}): {}", format_size(file_size), path_buf.display());
                        }
                    }
                    Err(_) => {
                        // File is likely locked/in-use; safely skip to avoid breaking running apps
                        stats.errors += 1;
                    }
                }
            } else {
                stats.files_deleted += 1;
                stats.bytes_freed += file_size;
                if verbose {
                    println!("[Dry-Run] Would remove file ({}): {}", format_size(file_size), path_buf.display());
                }
            }
        }
    }

    stats
}

fn flush_dns(dry_run: bool) {
    if dry_run {
        println!("[Dry-Run] Would flush DNS cache resolver.");
        return;
    }

    println!("Flushing DNS resolver cache...");
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("ipconfig")
            .arg("/flushdns")
            .output();
        match output {
            Ok(_) => println!("Successfully flushed DNS cache."),
            Err(e) => eprintln!("Failed to flush DNS cache: {}", e),
        }
    }

    #[cfg(target_os = "macos")]
    {
        let output1 = Command::new("dscacheutil").arg("-flushcache").output();
        let output2 = Command::new("killall").args(["-HUP", "mDNSResponder"]).output();
        if output1.is_ok() && output2.is_ok() {
            println!("Successfully flushed DNS cache.");
        } else {
            eprintln!("Failed to flush DNS cache completely.");
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try various common dns caching daemons/resolved services
        let systems = [
            ("resolvectl", vec!["flush-caches"]),
            ("systemd-resolve", vec!["--flush-caches"]),
            ("nscd", vec!["-i", "hosts"]),
        ];

        let mut success = false;
        for (cmd, args) in systems {
            if Command::new(cmd).args(&args).output().is_ok() {
                success = true;
                println!("Successfully flushed DNS cache using {}.", cmd);
                break;
            }
        }
        if !success {
            println!("No active local system DNS caching daemon found to flush (or missing privileges).");
        }
    }
}

fn drop_linux_caches(dry_run: bool) {
    #[cfg(target_os = "linux")]
    {
        if dry_run {
            println!("[Dry-Run] Would drop page cache, dentries, and inodes via /proc/sys/vm/drop_caches.");
            return;
        }

        println!("Requesting OS to drop page caches, dentries, and inodes (simulating reboot memory cleanup)...");
        // We write '3' to drop_caches. Needs root.
        match fs::write("/proc/sys/vm/drop_caches", "3") {
            Ok(_) => println!("Successfully dropped memory caches."),
            Err(e) => {
                println!("Note: Could not drop system memory caches ({}). Run with sudo/root to enable this.", e);
            }
        }
    }
}

fn print_help() {
    println!("Usage: clear-cache [OPTIONS]");
    println!();
    println!("Safely clears system and user cache/temp files and DNS without closing applications.");
    println!();
    println!("Options:");
    println!("  -d, --dry-run   Perform a trial run without deleting files");
    println!("  -v, --verbose   Show detailed logs of files/directories deleted");
    println!("  -h, --help      Display this help menu");
}

fn main() {
    let mut dry_run = false;
    let mut verbose = false;

    let args: Vec<String> = env::args().collect();
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-d" | "--dry-run" => dry_run = true,
            "-v" | "--verbose" => verbose = true,
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {
                eprintln!("Unknown option: {}", arg);
                print_help();
                std::process::exit(1);
            }
        }
    }

    println!("=== Soft Reboot & Cache Cleaner ===");
    if dry_run {
        println!("*** RUNNING IN DRY-RUN MODE (No files will be deleted) ***\n");
    }

    let mut total_stats = CleanStats::new();
    let mut targets: Vec<(String, PathBuf)> = Vec::new();

    // Setup platform specific clean targets
    #[cfg(target_os = "windows")]
    {
        if let Some(user_temp) = env::var_os("TEMP").map(PathBuf::from) {
            targets.push(("User Temp Directory".to_string(), user_temp));
        }
        targets.push(("System Temp Directory".to_string(), PathBuf::from("C:\\Windows\\Temp")));
        targets.push(("Windows Prefetch".to_string(), PathBuf::from("C:\\Windows\\Prefetch")));
        targets.push(("Windows Update Cache".to_string(), PathBuf::from("C:\\Windows\\SoftwareDistribution\\Download")));
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
            targets.push(("User Cache Directory".to_string(), home.join("Library/Caches")));
            targets.push(("User Logs Directory".to_string(), home.join("Library/Logs")));
        }
        targets.push(("System Cache Directory".to_string(), PathBuf::from("/Library/Caches")));
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
            targets.push(("User Cache Directory".to_string(), home.join(".cache")));
        }
        targets.push(("System Temp Directory".to_string(), PathBuf::from("/tmp")));
        targets.push(("System Var-Temp Directory".to_string(), PathBuf::from("/var/tmp")));
    }

    // Clean all folder targets
    for (name, path) in &targets {
        if path.exists() {
            println!("Cleaning {} ({})...", name, path.display());
            let stats = clean_directory(path, dry_run, verbose);
            total_stats.merge(&stats);
        } else if verbose {
            println!("Target path for {} does not exist: {}", name, path.display());
        }
    }

    // Flush DNS Cache
    flush_dns(dry_run);

    // Drop OS RAM Caches (Linux)
    drop_linux_caches(dry_run);

    println!("\n=== Clean Summary ===");
    println!("Files deleted:     {}", total_stats.files_deleted);
    println!("Folders deleted:   {}", total_stats.dirs_deleted);
    println!("Total space freed: {}", format_size(total_stats.bytes_freed));
    if total_stats.errors > 0 {
        println!("Locked/Skipped files (safely bypassed): {}", total_stats.errors);
    }
    println!("Clean completed successfully!");
}

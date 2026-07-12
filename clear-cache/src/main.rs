use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use std::env;
use std::io::{self, Write, BufRead};
use std::collections::HashMap;

struct CleanStats {
    files_deleted: u64,
    dirs_deleted: u64,
    bytes_freed: u64,
    errors: u64,
    skipped_inuse: u64,
}

impl CleanStats {
    fn new() -> Self {
        CleanStats {
            files_deleted: 0,
            dirs_deleted: 0,
            bytes_freed: 0,
            errors: 0,
            skipped_inuse: 0,
        }
    }

    fn merge(&mut self, other: &CleanStats) {
        self.files_deleted += other.files_deleted;
        self.dirs_deleted += other.dirs_deleted;
        self.bytes_freed += other.bytes_freed;
        self.errors += other.errors;
        self.skipped_inuse += other.skipped_inuse;
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

/// A process that has a file open: PID + human-readable label (comm name).
#[derive(Clone)]
struct Holder {
    pid: u32,
    label: String,
}

/// Scans /proc/<pid>/fd on Linux and builds a map of
/// open-file canonical path -> list of holders (PID + process name).
///
/// This is how we detect "<PID, APP> está usando cache" without external tools.
#[cfg(target_os = "linux")]
fn build_open_files_map() -> HashMap<PathBuf, Vec<Holder>> {
    let mut map: HashMap<PathBuf, Vec<Holder>> = HashMap::new();

    let proc = match fs::read_dir("/proc") {
        Ok(p) => p,
        Err(_) => return map,
    };

    for entry in proc.flatten() {
        let name = entry.file_name();
        let name = match name.to_str() {
            Some(n) => n,
            None => continue,
        };
        // Only numeric entries are PIDs.
        let pid: u32 = match name.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let label = read_proc_comm(pid).unwrap_or_else(|| "unknown".to_string());

        let fd_dir = format!("/proc/{pid}/fd");
        let fds = match fs::read_dir(&fd_dir) {
            Ok(f) => f,
            Err(_) => continue, // permission denied or process gone
        };

        for fd in fds.flatten() {
            // Each fd is a symlink to the real file it points at.
            if let Ok(target) = fs::read_link(fd.path()) {
                if target.is_absolute() {
                    map.entry(target)
                        .or_default()
                        .push(Holder { pid, label: label.clone() });
                }
            }
        }
    }

    map
}

#[cfg(not(target_os = "linux"))]
fn build_open_files_map() -> HashMap<PathBuf, Vec<Holder>> {
    HashMap::new()
}

#[cfg(target_os = "linux")]
fn read_proc_comm(pid: u32) -> Option<String> {
    let comm = fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
    let comm = comm.trim();
    if comm.is_empty() {
        None
    } else {
        Some(comm.to_string())
    }
}

/// Tracks which apps hold files, so we can ask about them once per app
/// instead of once per file.
struct InUseController {
    /// Canonical open-file path -> holders.
    open_files: HashMap<PathBuf, Vec<Holder>>,
    /// Decisions already made, keyed by a stable app identity (pid + label).
    /// true = user allowed deleting this app's files.
    decisions: HashMap<String, bool>,
    /// If true, never ask and never delete in-use files (non-interactive).
    assume_skip: bool,
}

impl InUseController {
    fn new(interactive: bool) -> Self {
        InUseController {
            open_files: build_open_files_map(),
            decisions: HashMap::new(),
            assume_skip: !interactive,
        }
    }

    /// Returns the holders of `path`, if any process currently has it open.
    fn holders_of(&self, path: &Path) -> Option<&Vec<Holder>> {
        // Resolve symlinks/relative bits so we compare canonical paths.
        let canon = fs::canonicalize(path).ok()?;
        self.open_files.get(&canon).filter(|v| !v.is_empty())
    }

    /// Decides whether an in-use file may be deleted, asking the user once
    /// per (grouped) app. Returns true if deletion is allowed.
    fn may_delete(&mut self, path: &Path, holders: &[Holder]) -> bool {
        // Build one stable key for this group of holders so we only ask once.
        let mut ids: Vec<String> = holders
            .iter()
            .map(|h| format!("{}:{}", h.pid, h.label))
            .collect();
        ids.sort();
        ids.dedup();
        let key = ids.join(",");

        if let Some(&decided) = self.decisions.get(&key) {
            return decided;
        }

        if self.assume_skip {
            self.decisions.insert(key, false);
            return false;
        }

        // Compose the "<PID, APP> está usando cache" prompt.
        let who = ids.join(", ");
        print!(
            "\n[{}] está usando: {}\n  Certeza que deseja apagar? [s/N] ",
            who,
            path.display()
        );
        let _ = io::stdout().flush();

        let mut answer = String::new();
        let allowed = match io::stdin().lock().read_line(&mut answer) {
            Ok(_) => {
                let a = answer.trim().to_lowercase();
                a == "s" || a == "sim" || a == "y" || a == "yes"
            }
            Err(_) => false,
        };

        self.decisions.insert(key, allowed);
        allowed
    }
}

fn clean_directory(
    path: &Path,
    dry_run: bool,
    verbose: bool,
    inuse: &mut InUseController,
) -> CleanStats {
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
            let sub_stats = clean_directory(&path_buf, dry_run, verbose, inuse);
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

            // Is this file currently held open by a running process?
            if let Some(holders) = inuse.holders_of(&path_buf).cloned() {
                if !inuse.may_delete(&path_buf, &holders) {
                    stats.skipped_inuse += 1;
                    if verbose {
                        println!("Skipped (in use): {}", path_buf.display());
                    }
                    continue;
                }
            }

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
        // Sync first so dirty pages are flushed before we drop caches.
        let _ = Command::new("sync").output();
        // We write '3' to drop_caches. Needs root.
        match fs::write("/proc/sys/vm/drop_caches", "3") {
            Ok(_) => println!("Successfully dropped memory caches."),
            Err(e) => {
                println!("Note: Could not drop system memory caches ({}). Run with sudo/root to enable this.", e);
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = dry_run;
    }
}

/// Clears error/system logs so the machine looks freshly booted.
/// On Linux this vacuums journald, truncates /var/log, and clears user logs.
fn clean_logs(dry_run: bool, verbose: bool, inuse: &mut InUseController) -> CleanStats {
    let mut stats = CleanStats::new();

    #[cfg(target_os = "linux")]
    {
        // 1. systemd journald
        if dry_run {
            println!("[Dry-Run] Would vacuum systemd journald logs (journalctl --vacuum-time=1s).");
        } else {
            println!("Vacuuming systemd journald logs...");
            match Command::new("journalctl").args(["--vacuum-time=1s"]).output() {
                Ok(o) if o.status.success() => println!("Successfully vacuumed journald logs."),
                Ok(_) => println!("Note: journald vacuum incomplete (needs sudo/root)."),
                Err(_) => println!("Note: journalctl not available."),
            }
        }

        // 2. /var/log — truncate active .log files, delete rotated ones.
        let var_log = PathBuf::from("/var/log");
        if var_log.exists() {
            println!("Cleaning error/system logs in {}...", var_log.display());
            let s = clean_log_dir(&var_log, dry_run, verbose, inuse);
            stats.merge(&s);
        }

        // 3. User-level logs (no root needed).
        if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
            let xsession = home.join(".xsession-errors");
            clean_single_log(&xsession, dry_run, verbose, inuse, &mut stats);
            clean_single_log(&home.join(".xsession-errors.old"), dry_run, verbose, inuse, &mut stats);

            let state = home.join(".local/state");
            if state.exists() {
                println!("Cleaning user state/logs in {}...", state.display());
                let s = clean_log_dir(&state, dry_run, verbose, inuse);
                stats.merge(&s);
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = (dry_run, verbose, &inuse);
    }

    stats
}

/// Walks a log directory: rotated logs (*.log.N, *.gz, *.old) get deleted,
/// active *.log files get truncated to 0 bytes (so open writers don't break),
/// and everything respects the in-use confirmation flow.
#[cfg(target_os = "linux")]
fn clean_log_dir(
    path: &Path,
    dry_run: bool,
    verbose: bool,
    inuse: &mut InUseController,
) -> CleanStats {
    let mut stats = CleanStats::new();

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return stats,
    };

    for entry in entries.flatten() {
        let p = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.is_dir() {
            let s = clean_log_dir(&p, dry_run, verbose, inuse);
            stats.merge(&s);
            continue;
        }

        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let is_log = name.ends_with(".log")
            || name.contains(".log.")
            || name.ends_with(".gz")
            || name.ends_with(".old")
            || name.ends_with(".1")
            || name.ends_with("-errors");
        if !is_log {
            continue;
        }

        clean_single_log(&p, dry_run, verbose, inuse, &mut stats);
    }

    stats
}

/// Deletes or truncates a single log file, respecting in-use confirmation.
fn clean_single_log(
    p: &Path,
    dry_run: bool,
    verbose: bool,
    inuse: &mut InUseController,
    stats: &mut CleanStats,
) {
    let meta = match fs::symlink_metadata(p) {
        Ok(m) => m,
        Err(_) => return,
    };
    if !meta.is_file() {
        return;
    }
    let size = meta.len();

    // Respect the "app is using this" confirmation.
    if let Some(holders) = inuse.holders_of(p).cloned() {
        if !inuse.may_delete(p, &holders) {
            stats.skipped_inuse += 1;
            if verbose {
                println!("Skipped log (in use): {}", p.display());
            }
            return;
        }
    }

    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
    // Active .log files: truncate to keep the file handle valid for writers.
    // Rotated/old logs: delete outright.
    let truncate_only = name.ends_with(".log") || name.ends_with("-errors");

    if dry_run {
        if truncate_only {
            println!("[Dry-Run] Would truncate log ({}): {}", format_size(size), p.display());
        } else {
            println!("[Dry-Run] Would remove log ({}): {}", format_size(size), p.display());
        }
        stats.bytes_freed += size;
        stats.files_deleted += 1;
        return;
    }

    if truncate_only {
        match fs::OpenOptions::new().write(true).truncate(true).open(p) {
            Ok(_) => {
                stats.bytes_freed += size;
                stats.files_deleted += 1;
                if verbose {
                    println!("Truncated log ({}): {}", format_size(size), p.display());
                }
            }
            Err(_) => stats.errors += 1,
        }
    } else {
        match fs::remove_file(p) {
            Ok(_) => {
                stats.bytes_freed += size;
                stats.files_deleted += 1;
                if verbose {
                    println!("Removed log ({}): {}", format_size(size), p.display());
                }
            }
            Err(_) => stats.errors += 1,
        }
    }
}

fn print_help(prog: &str) {
    println!("Usage: {prog} [OPTIONS]");
    println!();
    println!("Safely clears system and user cache/temp files, error logs, and DNS");
    println!("without closing applications. Files held open by a running process");
    println!("trigger a per-app confirmation prompt ('<PID, APP> está usando...').");
    println!();
    println!("Options:");
    println!("  -d, --dry-run     Perform a trial run without deleting files");
    println!("  -v, --verbose     Show detailed logs of files/directories deleted");
    println!("  -t, --temp        Clean temporary/temp directories only");
    println!("  -c, --cache       Clean user and system cache directories only");
    println!("  -l, --logs        Clean error/system logs only");
    println!("      --dns         Flush DNS resolver cache only");
    println!("      --ram         Drop OS memory caches only (Linux only, requires sudo/root)");
    println!("  -y, --yes         Non-interactive: skip all in-use files without asking");
    println!("  -h, --help        Display this help menu");
}

fn main() {
    let mut dry_run = false;
    let mut verbose = false;
    let mut clean_temp = false;
    let mut clean_cache = false;
    let mut clean_dns = false;
    let mut clean_ram = false;
    let mut clean_logs_flag = false;
    let mut non_interactive = false;

    let args: Vec<String> = env::args().collect();
    let prog = args.first().map(String::as_str).unwrap_or("clear-cache");

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-d" | "--dry-run" => dry_run = true,
            "-v" | "--verbose" => verbose = true,
            "-t" | "--temp" => clean_temp = true,
            "-c" | "--cache" => clean_cache = true,
            "-l" | "--logs" => clean_logs_flag = true,
            "-y" | "--yes" => non_interactive = true,
            "--dns" => clean_dns = true,
            "--ram" => clean_ram = true,
            "-h" | "--help" => {
                print_help(prog);
                return;
            }
            // Short flags — may be combined: -dvt etc.
            s if s.starts_with('-') && s.len() > 1 && !s.starts_with("--") => {
                for ch in s.chars().skip(1) {
                    match ch {
                        'd' => dry_run = true,
                        'v' => verbose = true,
                        't' => clean_temp = true,
                        'c' => clean_cache = true,
                        'l' => clean_logs_flag = true,
                        'y' => non_interactive = true,
                        'h' => {
                            print_help(prog);
                            return;
                        }
                        c => {
                            eprintln!("{prog}: invalid option -- '{c}'");
                            eprintln!("Try '{prog} --help' for more information.");
                            std::process::exit(1);
                        }
                    }
                }
            }
            other => {
                eprintln!("{prog}: unrecognized option '{other}'");
                eprintln!("Try '{prog} --help' for more information.");
                std::process::exit(1);
            }
        }
    }

    // Default to clean everything if no specific categories are selected
    let clean_all = !clean_temp && !clean_cache && !clean_dns && !clean_ram && !clean_logs_flag;
    let do_temp = clean_all || clean_temp;
    let do_cache = clean_all || clean_cache;
    let do_dns = clean_all || clean_dns;
    let do_ram = clean_all || clean_ram;
    let do_logs = clean_all || clean_logs_flag;

    println!("=== Soft Reboot & Cache Cleaner ===");
    if dry_run {
        println!("*** RUNNING IN DRY-RUN MODE (No files will be deleted) ***\n");
    }

    // Interactive unless -y was passed or we have no TTY isn't checked here;
    // -y forces non-interactive skip-in-use behavior.
    let mut inuse = InUseController::new(!non_interactive && !dry_run);

    let mut total_stats = CleanStats::new();
    let mut targets: Vec<(String, PathBuf)> = Vec::new();

    // Setup platform specific clean targets
    #[cfg(target_os = "windows")]
    {
        if do_cache {
            targets.push(("Windows Update Cache".to_string(), PathBuf::from("C:\\Windows\\SoftwareDistribution\\Download")));
        }
        if do_temp {
            if let Some(user_temp) = env::var_os("TEMP").map(PathBuf::from) {
                targets.push(("User Temp Directory".to_string(), user_temp));
            }
            targets.push(("System Temp Directory".to_string(), PathBuf::from("C:\\Windows\\Temp")));
            targets.push(("Windows Prefetch".to_string(), PathBuf::from("C:\\Windows\\Prefetch")));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
            if do_cache {
                targets.push(("User Cache Directory".to_string(), home.join("Library/Caches")));
            }
            if do_temp {
                targets.push(("User Logs Directory".to_string(), home.join("Library/Logs")));
            }
        }
        if do_cache {
            targets.push(("System Cache Directory".to_string(), PathBuf::from("/Library/Caches")));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
            if do_cache {
                targets.push(("User Cache Directory".to_string(), home.join(".cache")));
            }
        }
        if do_temp {
            targets.push(("System Temp Directory".to_string(), PathBuf::from("/tmp")));
            targets.push(("System Var-Temp Directory".to_string(), PathBuf::from("/var/tmp")));
        }
    }

    // Clean all folder targets
    for (name, path) in &targets {
        if path.exists() {
            println!("Cleaning {} ({})...", name, path.display());
            let stats = clean_directory(path, dry_run, verbose, &mut inuse);
            total_stats.merge(&stats);
        } else if verbose {
            println!("Target path for {} does not exist: {}", name, path.display());
        }
    }

    // Clean error/system logs
    if do_logs {
        let stats = clean_logs(dry_run, verbose, &mut inuse);
        total_stats.merge(&stats);
    }

    // Flush DNS Cache
    if do_dns {
        flush_dns(dry_run);
    }

    // Drop OS RAM Caches (Linux)
    if do_ram {
        drop_linux_caches(dry_run);
    }

    println!("\n=== Clean Summary ===");
    println!("Files deleted:     {}", total_stats.files_deleted);
    println!("Folders deleted:   {}", total_stats.dirs_deleted);
    println!("Total space freed: {}", format_size(total_stats.bytes_freed));
    if total_stats.skipped_inuse > 0 {
        println!("In-use files kept (declined):           {}", total_stats.skipped_inuse);
    }
    if total_stats.errors > 0 {
        println!("Locked/Skipped files (safely bypassed): {}", total_stats.errors);
    }
    println!("Clean completed successfully!");
}

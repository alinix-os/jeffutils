use std::env;
use std::fs;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const SKIP_FS: &[&str] = &["proc", "sysfs", "devpts", "tmpfs", "cgroup", "cgroup2", "pstore", "securityfs", "debugfs", "tracefs", "fusectl", "configfs", "hugetlbfs", "mqueue", "binfmt_misc", "autofs", "rpc_pipefs", "nfsd"];

fn human_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}G", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1}M", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

fn print_usage() {
    println!("diskfree {} - display disk space usage", VERSION);
    println!();
    println!("Usage: diskfree [OPTIONS] [FILESYSTEM...]");
    println!();
    println!("Options:");
    println!("  -h, --help       display this help message");
    println!("  -v, --version    display version");
    println!("  -T TYPE          filter by filesystem type");
}

fn parse_args() -> (Vec<String>, Option<String>) {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut filter_type: Option<String> = None;
    let mut filesystems = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "-v" | "--version" => {
                println!("diskfree {}", VERSION);
                std::process::exit(0);
            }
            "-T" => {
                i += 1;
                if i < args.len() {
                    filter_type = Some(args[i].clone());
                } else {
                    eprintln!("diskfree: option requires an argument -- 'T'");
                    std::process::exit(1);
                }
            }
            other => {
                filesystems.push(other.to_string());
            }
        }
        i += 1;
    }
    (filesystems, filter_type)
}

fn main() {
    let (filesystems, filter_type) = parse_args();

    let mounts_content = fs::read_to_string("/proc/mounts").unwrap_or_else(|e| {
        eprintln!("diskfree: cannot read /proc/mounts: {}", e);
        std::process::exit(1);
    });

    let mut total_size: u64 = 0;
    let mut total_used: u64 = 0;
    let mut total_avail: u64 = 0;

    println!("{:<20} {:>10} {:>10} {:>10} {:>5} {:<}", "Filesystem", "Size", "Used", "Avail", "Use%", "Mounted on");

    for line in mounts_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let device = parts[0];
        let mount_point = parts[1];
        let fs_type = parts[2];

        if SKIP_FS.contains(&fs_type) {
            continue;
        }

        if let Some(ref ft) = filter_type {
            if fs_type != ft.as_str() {
                continue;
            }
        }

        if filesystems.len() > 0 && !filesystems.iter().any(|f| f == mount_point || f == device) {
            continue;
        }

        unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            let c_mount = std::ffi::CString::new(mount_point).unwrap();
            if libc::statvfs(c_mount.as_ptr(), &mut stat) != 0 {
                continue;
            }

            let block_size = stat.f_frsize as u64;
            if block_size == 0 {
                continue;
            }

            let total = stat.f_blocks * block_size;
            let free = stat.f_bfree * block_size;
            let avail = stat.f_bavail * block_size;
            let used = total - free;

            let use_pct = if total > 0 {
                ((used as f64 / total as f64) * 100.0) as u64
            } else {
                0
            };

            total_size += total;
            total_used += used;
            total_avail += avail;

            println!("{:<20} {:>10} {:>10} {:>10} {:>4}% {:<}", device, human_size(total), human_size(used), human_size(avail), use_pct, mount_point);
        }
    }

    println!("{:<20} {:>10} {:>10} {:>10} {:>5} {:<}", "total", human_size(total_size), human_size(total_used), human_size(total_avail), if total_size > 0 { ((total_used as f64 / total_size as f64) * 100.0) as u64 } else { 0 }, "");
}

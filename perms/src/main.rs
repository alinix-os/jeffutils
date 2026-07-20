use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

fn print_usage() {
    eprintln!("Usage: {} <caminho> [ação] [args...]", std::env::args().nth(0).unwrap_or_else(|| "perms".into()));
}

fn describe_error(e: &std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::NotFound => "File not found".into(),
        std::io::ErrorKind::PermissionDenied => "Do not have permissions".into(),
        std::io::ErrorKind::InvalidInput => "Invalid input".into(),
        _ => format!("Error: {}", e),
    }
}

fn format_triple(mode: u32) -> String {
    let r = if mode & 4 != 0 { "r" } else { "-" };
    let w = if mode & 2 != 0 { "w" } else { "-" };
    let x = if mode & 1 != 0 { "x" } else { "-" };
    format!("{}{}{}", r, w, x)
}

fn get_username(uid: u32) -> String {
    std::fs::read_to_string("/etc/passwd")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.split(':').nth(2).and_then(|id| id.parse::<u32>().ok()) == Some(uid))
                .and_then(|l| l.split(':').next())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| uid.to_string())
}

fn get_groupname(gid: u32) -> String {
    std::fs::read_to_string("/etc/group")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.split(':').nth(2).and_then(|id| id.parse::<u32>().ok()) == Some(gid))
                .and_then(|l| l.split(':').next())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| gid.to_string())
}

fn show_perms(path: &Path) {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", describe_error(&e));
            std::process::exit(1);
        }
    };

    let uid = metadata.uid();
    let gid = metadata.gid();
    let username = get_username(uid);
    let groupname = get_groupname(gid);

    println!("Owner : {}", username);
    println!("Group : {}", groupname);

    let mode = metadata.permissions().mode();
    let owner = format_triple((mode >> 6) & 7);
    let group = format_triple((mode >> 3) & 7);
    let other = format_triple(mode & 7);

    println!();
    println!("Permissions");
    println!();
    println!("Owner : {}", owner);
    println!("Group : {}", group);
    println!("Others: {}", other);
    println!();

    let protected = (mode & 0o1000) != 0;
    println!("Protected : {}", if protected { "Yes" } else { "No" });
    let immutable = std::process::Command::new("lsattr")
        .arg(path.as_os_str())
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.lines().next().map(|first| {
                    first.split_whitespace().next().map(|flags| flags.contains('i')).unwrap_or(false)
                })
            } else {
                None
            }
        });
    match immutable {
        Some(true) => println!("Immutable : Yes"),
        Some(false) => println!("Immutable : No"),
        None => println!("Immutable : Unknown"),
    }
}

fn set_perms(path: &Path, mode_str: &str) {
    let mode = parse_mode(mode_str).unwrap_or_else(|| {
        eprintln!("Error: invalid permission string '{}'", mode_str);
        std::process::exit(1);
    });

    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", describe_error(&e));
            std::process::exit(1);
        }
    };
    let mut perms = metadata.permissions();
    perms.set_mode(mode);
    if let Err(e) = fs::set_permissions(path, perms) {
        eprintln!("{}", describe_error(&e));
        std::process::exit(1);
    }
    println!("Permissions updated for {}", path.display());
}

fn parse_mode(s: &str) -> Option<u32> {
    if s.len() == 9 || s.len() == 10 {
        let start = if s.len() == 10 { 1 } else { 0 };
        let owner = parse_triple(&s[start..start + 3])?;
        let group = parse_triple(&s[start + 3..start + 6])?;
        let other = parse_triple(&s[start + 6..start + 9])?;
        Some((owner << 6) | (group << 3) | other)
    } else if s.len() == 3 || s.len() == 4 {
        u32::from_str_radix(s, 8).ok()
    } else {
        None
    }
}

fn parse_triple(s: &str) -> Option<u32> {
    if s.len() != 3 {
        return None;
    }
    let bytes = s.as_bytes();
    let r = if bytes[0] == b'r' { 4 } else { 0 };
    let w = if bytes[1] == b'w' { 2 } else { 0 };
    let x = if bytes[2] == b'x' { 1 } else { 0 };
    Some(r + w + x)
}

fn set_owner(path: &Path, owner: &str) {
    let status = std::process::Command::new("chown")
        .arg(owner)
        .arg(path.as_os_str())
        .status();
    match status {
        Ok(s) if s.success() => println!("Owner of {} set to {}", path.display(), owner),
        Ok(_) => {
            eprintln!("Error: could not change owner (permission denied)");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn set_group(path: &Path, group: &str) {
    let status = std::process::Command::new("chgrp")
        .arg(group)
        .arg(path.as_os_str())
        .status();
    match status {
        Ok(s) if s.success() => println!("Group of {} set to {}", path.display(), group),
        Ok(_) => {
            eprintln!("Error: could not change group (permission denied)");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn make_executable(path: &Path) {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", describe_error(&e));
            std::process::exit(1);
        }
    };
    let mut perms = metadata.permissions();
    let mode = perms.mode();
    perms.set_mode(mode | 0o111);
    if let Err(e) = fs::set_permissions(path, perms) {
        eprintln!("{}", describe_error(&e));
        std::process::exit(1);
    }
    println!("{} is now executable", path.display());
}

fn make_readonly(path: &Path) {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", describe_error(&e));
            std::process::exit(1);
        }
    };
    let mut perms = metadata.permissions();
    perms.set_mode(perms.mode() & !0o222);
    if let Err(e) = fs::set_permissions(path, perms) {
        eprintln!("{}", describe_error(&e));
        std::process::exit(1);
    }
    println!("{} is now read-only", path.display());
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("perms", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    for arg in &args {
        if arg == "--help" || arg == "-h" {
            print_usage();
            println!("Manages file permissions, ownership, and ACL.");
            println!("  set <mode>         Set permissions (e.g., rw-r--r--, 644)");
            println!("  owner <user>       Change owner");
            println!("  group <group>      Change group");
            println!("  exec               Make executable");
            println!("  readonly           Make read-only");
            println!("  protect            Protect file (kernel-level)");
            println!("  unprotect          Remove protection");
            println!("  allow <user> <rw>  Grant ACL access");
            println!("  deny <user>        Deny ACL access");
            println!("  --recursive        Apply recursively");
            println!("  --help, -h         Show this help message");
            println!("  --version          Show version information");
            return;
        }
        if arg == "--version" {
            println!("perms version 0.1.0");
            return;
        }
    }

    let path = Path::new(&args[0]);
    if !path.exists() {
        eprintln!("Error: path '{}' not found", args[0]);
        std::process::exit(1);
    }

    if args.len() == 1 {
        show_perms(path);
        return;
    }

    let action = &args[1];
    match action.as_str() {
        "set" => {
            if args.len() < 3 {
                eprintln!("Error: set requires a permission string");
                std::process::exit(1);
            }
            set_perms(path, &args[2]);
        }
        "owner" | "chown" => {
            if args.len() < 3 {
                eprintln!("Error: owner requires a username");
                std::process::exit(1);
            }
            set_owner(path, &args[2]);
        }
        "group" | "chgrp" => {
            if args.len() < 3 {
                eprintln!("Error: group requires a group name");
                std::process::exit(1);
            }
            set_group(path, &args[2]);
        }
        "exec" => make_executable(path),
        "readonly" => make_readonly(path),
        "protect" => {
            let status = std::process::Command::new("chattr")
                .args(["+i", path.to_str().unwrap()])
                .status();
            match status {
                Ok(s) if s.success() => println!("Protected: {} is now protected (kernel-level)", path.display()),
                Ok(_) => {
                    eprintln!("Error: could not set immutable flag (permission denied)");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Error: chattr not available or failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "unprotect" => {
            let status = std::process::Command::new("chattr")
                .args(["-i", path.to_str().unwrap()])
                .status();
            match status {
                Ok(s) if s.success() => println!("Unprotected: {} protection removed", path.display()),
                Ok(_) => {
                    eprintln!("Error: could not remove immutable flag (permission denied)");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Error: chattr not available or failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "allow" => {
            if args.len() < 4 {
                eprintln!("Error: allow requires <user> <permissions>");
                std::process::exit(1);
            }
            let user = &args[2];
            let perms = &args[3];
            let acl_spec = format!("u:{}:{}", user, perms);
            let status = std::process::Command::new("setfacl")
                .args(["-m", &acl_spec, path.to_str().unwrap()])
                .status();
            match status {
                Ok(s) if s.success() => println!("ACL: {} granted {} access to {}", user, perms, path.display()),
                Ok(_) => {
                    eprintln!("Error: could not set ACL (permission denied)");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Error: setfacl not available or failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "deny" => {
            if args.len() < 3 {
                eprintln!("Error: deny requires a username");
                std::process::exit(1);
            }
            let user = &args[2];
            let acl_spec = format!("u:{}:---", user);
            let status = std::process::Command::new("setfacl")
                .args(["-m", &acl_spec, path.to_str().unwrap()])
                .status();
            match status {
                Ok(s) if s.success() => println!("ACL: {} denied access to {}", user, path.display()),
                Ok(_) => {
                    eprintln!("Error: could not set ACL (permission denied)");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Error: setfacl not available or failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Error: unknown action '{}'", action);
            std::process::exit(1);
        }
    }
}

use std::collections::HashMap;
use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::exit;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn mappings() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    // aliases already created: touch, mkdir are not included since they're separate crates
    m.insert("cat", "read");
    m.insert("chmod", "perms");
    m.insert("chown", "perms");
    m.insert("chgrp", "perms");
    m.insert("cp", "cp");
    m.insert("rm", "remove");
    m.insert("rmdir", "remove");
    m.insert("mv", "rename");
    m.insert("ln", "link");
    m.insert("ls", "ls");
    m.insert("find", "find");
    m.insert("stat", "stat");
    m.insert("date", "clock");
    m.insert("sleep", "sleep");
    m.insert("uptime", "uptime");
    m.insert("kill", "kill");
    m.insert("nice", "nice");
    m.insert("ps", "ps");
    m.insert("pwd", "pwd");
    m.insert("whoami", "whoami");
    m.insert("groups", "groups");
    m.insert("head", "head");
    m.insert("tail", "tail");
    m.insert("wc", "wc");
    m.insert("echo", "echo");
    m.insert("env", "env");
    m.insert("mount", "mount");
    m.insert("umount", "umount");
    m.insert("uname", "kinfo");
    m.insert("hostname", "sysinfo");
    m.insert("free", "memory");
    m.insert("sort", "arrange");
    m.insert("uniq", "dedup");
    m.insert("cut", "slice");
    m.insert("tr", "convert");
    m.insert("tac", "flip");
    m.insert("nl", "number");
    m.insert("paste", "stitch");
    m.insert("split", "chunk");
    m.insert("tee", "mirror");
    m.insert("df", "diskfree");
    m.insert("du", "spaceused");
    m.insert("dd", "blockcopy");
    m.insert("shred", "destroy");
    m.insert("truncate", "resize");
    m.insert("basename", "leaf");
    m.insert("dirname", "stem");
    m.insert("realpath", "resolve");
    m.insert("readlink", "dereference");
    m.insert("id", "identity");
    m.insert("who", "online");
    m.insert("nproc", "cpucount");
    m.insert("yes", "repeat");
    m.insert("seq", "countup");
    m.insert("od", "bytedump");
    m.insert("expr", "calculate");
    m.insert("base64", "encode64");
    m.insert("base32", "encode32");
    m.insert("cksum", "checksum");
    m.insert("b2sum", "blake2");
    m.insert("sum", "crcsum");
    m.insert("md5sum", "hash");
    m.insert("sha1sum", "hash");
    m.insert("sha256sum", "hash");
    m.insert("sha512sum", "hash");
    m.insert("mktemp", "temppath");
    m.insert("install", "deploy");
    m.insert("sync", "flush");
    m.insert("nohup", "persist");
    m.insert("tty", "terminal");
    m.insert("printf", "format");
    m.insert("fold", "wrap");
    m.insert("fmt", "reflow");
    m.insert("join", "meld");
    m.insert("comm", "compare");
    m.insert("factor", "primegen");
    m.insert("tsort", "toposort");
    m.insert("pr", "paginate");
    m.insert("expand", "untab");
    m.insert("unexpand", "retab");
    m.insert("numfmt", "unitformat");
    m.insert("csplit", "segment");
    m.insert("ptx", "permutext");
    m.insert("pathchk", "pathcheck");
    m.insert("mkfifo", "pipefile");
    m.insert("mknod", "devnode");
    m.insert("hostid", "machineid");
    m.insert("logname", "sessionuser");
    m.insert("users", "sessions");
    m.insert("pinky", "usercheck");
    m.insert("chcon", "context");
    m.insert("runcon", "conrun");
    m.insert("chroot", "jail");
    m.insert("stty", "termconfig");
    m.insert("dircolors", "dirtheme");
    m.insert("touch", "touch");
    m.insert("mkdir", "mkdir");
    m.insert("grep", "search");
    m.insert("tree", "tree");
    m
}

fn print_usage() {
    eprintln!("cutils v{VERSION} - jeffutils coreutils alias manager\n");
    eprintln!("Usage: cutils <COMMAND> [OPTIONS]\n");
    eprintln!("Commands:");
    eprintln!("  install     Create symlinks for all coreutils names");
    eprintln!("  uninstall   Remove all coreutils symlinks");
    eprintln!("  list        Show all name mappings");
    eprintln!("  status      Check which symlinks are installed");
    eprintln!("  which NAME  Show which jeffutils command a coreutils name maps to\n");
    eprintln!("Options:");
    eprintln!("  -d DIR      Target directory for symlinks (default: /usr/local/bin)");
    eprintln!("  --help      Show this help");
    eprintln!("  --version   Show version");
}

fn get_bin_dir() -> PathBuf {
    PathBuf::from("/usr/local/bin")
}

fn find_jeffutils_binary(name: &str) -> Option<PathBuf> {
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    if let Ok(path_var) = env::var("PATH") {
        for dir in path_var.split(':') {
            let candidate = PathBuf::from(dir).join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

fn cmd_install(target: &Path) {
    let map = mappings();
    let mut created = 0;
    let mut skipped = 0;
    let mut not_found = Vec::new();

    let target = if !target.exists() {
        match fs::create_dir_all(target) {
            Ok(_) => target,
            Err(e) => {
                eprintln!("Error: cannot create directory {}: {}", target.display(), e);
                exit(1);
            }
        }
    } else {
        target
    };

    for (alias, real) in map {
        let link_path = target.join(alias);
        match find_jeffutils_binary(real) {
            Some(bin_path) => {
                if link_path.exists() || link_path.symlink_metadata().is_ok() {
                    skipped += 1;
                    continue;
                }
                match symlink(&bin_path, &link_path) {
                    Ok(_) => created += 1,
                    Err(e) => {
                        eprintln!("Warning: cannot create {}: {}", link_path.display(), e);
                    }
                }
            }
            None => {
                not_found.push(alias);
            }
        }
    }

    println!("Created {created} symlinks, skipped {skipped} (already exist)");
    if !not_found.is_empty() {
        eprintln!("Warning: {} binaries not found: {}", not_found.len(), not_found.join(", "));
    }
}

fn cmd_uninstall(target: &Path) {
    let map = mappings();
    let mut removed = 0;
    let mut not_found = 0;

    for (&alias, _) in &map {
        let link_path = target.join(alias);
        if let Ok(meta) = link_path.symlink_metadata() {
            if meta.file_type().is_symlink() {
                if fs::remove_file(&link_path).is_ok() {
                    removed += 1;
                }
            }
        } else {
            not_found += 1;
        }
    }

    println!("Removed {removed} symlinks ({not_found} were not present)");
}

fn cmd_list() {
    let map = mappings();
    let mut entries: Vec<_> = map.into_iter().collect();
    entries.sort_by_key(|(alias, _)| *alias);

    let max_alias = entries.iter().map(|(a, _)| a.len()).max().unwrap_or(0);
    println!("{:<width$}  jeffutils command", "coreutils name", width = max_alias + 2);
    println!("{}", "-".repeat(max_alias + 2 + 20));
    for (alias, real) in &entries {
        println!("{:<width$}  {real}", alias, width = max_alias + 2);
    }
    println!("\n{} mappings total", entries.len());
}

fn cmd_status(target: &Path) {
    let map = mappings();
    let mut installed = 0;
    let mut missing = 0;

    let mut entries: Vec<_> = map.into_iter().collect();
    entries.sort_by_key(|(alias, _)| *alias);

    for (alias, real) in &entries {
        let link_path = target.join(alias);
        let exists = link_path.symlink_metadata().is_ok();
        let marker = if exists { "[ok]" } else { "[--]" };
        println!("{marker} {alias:<20} -> {real}");
        if exists { installed += 1; } else { missing += 1; }
    }

    println!("\n{installed} installed, {missing} missing");
}

fn cmd_which(name: &str) {
    let map = mappings();
    match map.get(name) {
        Some(real) => {
            println!("{name} -> {real}");
            match find_jeffutils_binary(real) {
                Some(p) => println!("  binary: {}", p.display()),
                None => println!("  binary: not found in PATH"),
            }
        }
        None => {
            eprintln!("Unknown coreutils name: {name}");
            exit(1);
        }
    }
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("cutils", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        exit(1);
    }

    let mut target_dir = get_bin_dir();
    let mut cmd = None;
    let mut cmd_arg = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => { print_usage(); exit(0); }
            "--version" | "-v" => { println!("cutils v{VERSION}"); exit(0); }
            "-d" => {
                i += 1;
                if let Some(d) = args.get(i) {
                    target_dir = PathBuf::from(d);
                } else {
                    eprintln!("Error: -d requires a directory argument");
                    exit(1);
                }
            }
            "install" | "uninstall" | "list" | "status" | "which" => {
                cmd = Some(args[i].clone());
            }
            _ => {
                if cmd.is_none() {
                    eprintln!("Unknown command: {}", args[i]);
                    print_usage();
                    exit(1);
                } else if cmd.as_deref() == Some("which") {
                    cmd_arg = Some(args[i].clone());
                }
            }
        }
        i += 1;
    }

    match cmd.as_deref() {
        Some("install") => cmd_install(&target_dir),
        Some("uninstall") => cmd_uninstall(&target_dir),
        Some("list") => cmd_list(),
        Some("status") => cmd_status(&target_dir),
        Some("which") => {
            match cmd_arg {
                Some(name) => cmd_which(&name),
                None => {
                    eprintln!("Error: 'which' requires a name argument");
                    exit(1);
                }
            }
        }
        _ => {
            print_usage();
            exit(1);
        }
    }
}

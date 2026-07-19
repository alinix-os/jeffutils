use std::collections::HashMap;
use std::env;
use std::fs;

const VERSION: &str = "0.1.0";

fn print_usage() {
    let name = env::args().next().unwrap_or_else(|| "identity".into());
    eprintln!("Usage: {name} [-u] [-g] [-G] [-n] [-r] [-e]");
    eprintln!("Print real and effective user and group IDs.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -u          print only the effective user ID");
    eprintln!("  -g          print only the effective group ID");
    eprintln!("  -G          print all group IDs");
    eprintln!("  -n          print names instead of numbers (for use with -u, -g, -G)");
    eprintln!("  -r          print only the real ID (not effective)");
    eprintln!("  -e          print only the effective ID (default)");
    eprintln!("  -h, --help  show this help message");
    eprintln!("  -v, --version show version");
}

fn read_passwd_map() -> HashMap<u32, String> {
    let mut map = HashMap::new();
    if let Ok(content) = fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(uid) = parts[2].parse::<u32>() {
                    map.insert(uid, parts[0].to_string());
                }
            }
        }
    }
    map
}

fn read_group_map() -> HashMap<u32, String> {
    let mut map = HashMap::new();
    if let Ok(content) = fs::read_to_string("/etc/group") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(gid) = parts[2].parse::<u32>() {
                    map.insert(gid, parts[0].to_string());
                }
            }
        }
    }
    map
}

fn get_group_ids() -> Vec<u32> {
    unsafe {
        let gid = libc::getgid();
        let mut groups = vec![gid as u32];
        let ngroups: libc::c_int = libc::getgroups(0, std::ptr::null_mut());
        if ngroups > 0 {
            let mut buf = vec![0 as libc::gid_t; ngroups as usize];
            libc::getgroups(ngroups, buf.as_mut_ptr());
            for &g in &buf {
                groups.push(g as u32);
            }
        }
        groups.sort();
        groups.dedup();
        groups
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "-v" || arg == "--version" {
            println!("identity {VERSION}");
            return;
        }
    }

    let mut opt_user = false;
    let mut opt_group = false;
    let mut opt_groups = false;
    let mut opt_names = false;
    let mut opt_real = false;

    for arg in &args {
        match arg.as_str() {
            "-u" => opt_user = true,
            "-g" => opt_group = true,
            "-G" => opt_groups = true,
            "-n" => opt_names = true,
            "-r" => opt_real = true,
            "-e" => {} // effective is the default behavior
            other if other.starts_with('-') && other.len() > 1 && !other.starts_with("--") => {
                for ch in other[1..].chars() {
                    match ch {
                        'u' => opt_user = true,
                        'g' => opt_group = true,
                        'G' => opt_groups = true,
                        'n' => opt_names = true,
                        'r' => opt_real = true,
                        'e' => {} // effective is the default
                        _ => {
                            eprintln!("identity: invalid option '--{ch}'");
                            std::process::exit(1);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let passwd_map = read_passwd_map();
    let group_map = read_group_map();

    let ruid = unsafe { libc::getuid() } as u32;
    let euid = unsafe { libc::geteuid() } as u32;
    let rgid = unsafe { libc::getgid() } as u32;
    let egid = unsafe { libc::getegid() } as u32;

    if opt_groups {
        let groups = get_group_ids();
        if opt_names {
            let names: Vec<String> = groups
                .iter()
                .map(|g| group_map.get(g).cloned().unwrap_or_else(|| g.to_string()))
                .collect();
            println!("{}", names.join(" "));
        } else {
            let strs: Vec<String> = groups.iter().map(|g| g.to_string()).collect();
            println!("{}", strs.join(" "));
        }
        return;
    }

    if opt_user {
        let uid = if opt_real { ruid } else { euid };
        if opt_names {
            let name = passwd_map.get(&uid).cloned().unwrap_or_else(|| uid.to_string());
            println!("{name}");
        } else {
            println!("{uid}");
        }
        return;
    }

    if opt_group {
        let gid = if opt_real { rgid } else { egid };
        if opt_names {
            let name = group_map.get(&gid).cloned().unwrap_or_else(|| gid.to_string());
            println!("{name}");
        } else {
            println!("{gid}");
        }
        return;
    }

    // Default: print uid=gid=ruid=rgid format like `id`
    let uid = if opt_real { ruid } else { euid };
    let gid = if opt_real { rgid } else { egid };

    if opt_names {
        let uname = passwd_map.get(&uid).cloned().unwrap_or_else(|| uid.to_string());
        let gname = group_map.get(&gid).cloned().unwrap_or_else(|| gid.to_string());
        println!("uid={uname}({uid}) gid={gname}({gid})");
    } else {
        println!("uid={uid} gid={gid}");
    }
}

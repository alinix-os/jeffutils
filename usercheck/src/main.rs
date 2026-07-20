use std::env;
use std::fs;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("usercheck - lightweight user info query");
    eprintln!();
    eprintln!("USAGE: usercheck [-l|-s|-w] [USER...]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -l          long format (login name, real name, terminal, idle, login time, host)");
    eprintln!("  -s          short format (login name, name, terminal, host)");
    eprintln!("  -w          wide format");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
    eprintln!();
    eprintln!("Default: login name, name, terminal");
}

struct UserInfo {
    login: String,
    real_name: String,
    home: String,
    shell: String,
}

fn read_passwd() -> Vec<UserInfo> {
    let content = fs::read_to_string("/etc/passwd").unwrap_or_default();
    content
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 7 {
                Some(UserInfo {
                    login: fields[0].to_string(),
                    real_name: fields[4].to_string(),
                    home: fields[5].to_string(),
                    shell: fields[6].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn get_terminal(user: &str) -> String {
    let utmp_path = "/var/run/utmp";
    let content = fs::read_to_string(utmp_path).unwrap_or_default();

    for entry in content.bytes().collect::<Vec<u8>>().chunks(384) {
        if entry.len() < 384 {
            continue;
        }
        let ut_type = u16::from_ne_bytes([entry[0], entry[1]]);
        if ut_type != 7 {
            continue;
        }
        let user_bytes = &entry[4..36];
        let user_str = std::ffi::CStr::from_bytes_with_nul(user_bytes)
            .ok()
            .and_then(|c| c.to_str().ok())
            .unwrap_or("")
            .trim();
        if user_str == user {
            let line_bytes = &entry[36..44];
            let line_str = std::ffi::CStr::from_bytes_with_nul(line_bytes)
                .ok()
                .and_then(|c| c.to_str().ok())
                .unwrap_or("")
                .trim();
            return line_str.to_string();
        }
    }

    String::new()
}

fn format_entry(info: &UserInfo, terminal: &str, format: &str) -> String {
    match format {
        "long" => {
            format!(
                "{:<16} {:<20} {:<8} {:<6} {:<19} {}",
                info.login, info.real_name, terminal, "-", "-", info.home
            )
        }
        "short" => {
            format!("{:<16} {:<20} {:<8} {}", info.login, info.real_name, terminal, info.home)
        }
        "wide" => {
            format!(
                "{:<16} {:<20} {:<8} {}",
                info.login, info.real_name, terminal, info.shell
            )
        }
        _ => {
            format!("{:<16} {:<20} {:<8}", info.login, info.real_name, terminal)
        }
    }
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("usercheck", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    let mut format = "default";
    let mut users: Vec<String> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("usercheck {}", VERSION);
                process::exit(0);
            }
            "-l" => format = "long",
            "-s" => format = "short",
            "-w" => format = "wide",
            _ if args[i].starts_with('-') => {
                eprintln!("usercheck: unknown option '{}'", args[i]);
                process::exit(2);
            }
            _ => {
                users.push(args[i].clone());
            }
        }
        i += 1;
    }

    let passwd = read_passwd();

    if users.is_empty() {
        let uid = unsafe { libc::getuid() };
        let passwd_content = fs::read_to_string("/etc/passwd").unwrap_or_default();
        for line in passwd_content.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 3 {
                if let Ok(file_uid) = fields[2].parse::<u32>() {
                    if file_uid == uid {
                        users.push(fields[0].to_string());
                        break;
                    }
                }
            }
        }
    }

    for user in &users {
        if let Some(info) = passwd.iter().find(|u| u.login == *user) {
            let terminal = get_terminal(user);
            println!("{}", format_entry(info, &terminal, format));
        } else {
            eprintln!("usercheck: unknown user '{}'", user);
        }
    }
}

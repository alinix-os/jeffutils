use std::env;
use std::fs;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("sessionuser - print login name of current user");
    eprintln!();
    eprintln!("USAGE: sessionuser [OPTIONS]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
}

fn get_login_name() -> Result<String, String> {
    if let Ok(name) = env::var("LOGNAME") {
        if !name.is_empty() {
            return Ok(name);
        }
    }

    if let Ok(name) = env::var("USER") {
        if !name.is_empty() {
            return Ok(name);
        }
    }

    let uid = unsafe { libc::getuid() };

    let passwd_content = fs::read_to_string("/etc/passwd").map_err(|e| {
        format!("cannot read /etc/passwd: {}", e)
    })?;

    for line in passwd_content.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() >= 3 {
            if let Ok(file_uid) = fields[2].parse::<u32>() {
                if file_uid == uid {
                    return Ok(fields[0].to_string());
                }
            }
        }
    }

    Err(format!("cannot find user with UID {}", uid))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("sessionuser {}", VERSION);
                process::exit(0);
            }
            _ => {
                eprintln!("sessionuser: unknown option '{}'", arg);
                process::exit(2);
            }
        }
    }

    match get_login_name() {
        Ok(name) => println!("{}", name),
        Err(e) => {
            eprintln!("sessionuser: {}", e);
            process::exit(1);
        }
    }
}

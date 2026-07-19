use std::env;
use std::process::Command;

fn print_usage() {
    eprintln!("Usage: env [OPTION]... [-] [NAME=VALUE]... [COMMAND [ARG]...]");
    eprintln!("Set each NAME to VALUE in the environment and run COMMAND.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -i, --ignore-environment   start with an empty environment");
    eprintln!("  -h, --help                 display this help and exit");
    eprintln!("      --version              output version information and exit");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut clear_env = false;
    let mut vars_to_set = Vec::new();
    let mut cmd_start_idx = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "--version" {
            println!("env (JeffUtils) 1.0");
            return;
        }

        if arg == "-i" || arg == "--ignore-environment" {
            clear_env = true;
        } else if arg == "-" {
            clear_env = true;
        } else if arg.contains('=') {
            vars_to_set.push(arg.clone());
        } else {
            cmd_start_idx = Some(i);
            break;
        }
        i += 1;
    }

    let mut command = if let Some(idx) = cmd_start_idx {
        let cmd_name = &args[idx];
        let cmd_args = &args[idx + 1..];
        let mut c = Command::new(cmd_name);
        c.args(cmd_args);
        Some(c)
    } else {
        None
    };

    if clear_env {
        if let Some(ref mut c) = command {
            c.env_clear();
        } else {
            return;
        }
    }

    let mut custom_vars = Vec::new();
    for var_def in &vars_to_set {
        if let Some(pos) = var_def.find('=') {
            let name = &var_def[..pos];
            let val = &var_def[pos + 1..];
            custom_vars.push((name.to_string(), val.to_string()));
            if let Some(ref mut c) = command {
                c.env(name, val);
            }
        }
    }

    if let Some(mut c) = command {
        let status = match c.status() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("env: failed to execute command: {}", e);
                std::process::exit(127);
            }
        };
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            match status.code() {
                Some(code) => std::process::exit(code),
                None => {
                    let signal = status.signal().unwrap_or(1);
                    std::process::exit(128 + signal);
                }
            }
        }
        #[cfg(not(unix))]
        std::process::exit(status.code().unwrap_or(0));
    } else {
        if clear_env {
            for (k, v) in custom_vars {
                println!("{}={}", k, v);
            }
        } else {
            let mut current_vars: std::collections::HashMap<String, String> = env::vars().collect();
            for (k, v) in custom_vars {
                current_vars.insert(k, v);
            }
            for (k, v) in current_vars {
                println!("{}={}", k, v);
            }
        }
    }
}

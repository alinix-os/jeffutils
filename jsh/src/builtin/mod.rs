use crate::shell::ShellState;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn run_jeofetch() {
    let status = Command::new("jeofetch")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    if let Err(e) = status {
        eprintln!("jsh: jeofetch: {}", e);
    }
}

pub fn is_builtin(cmd: &str) -> bool {
    matches!(cmd, "cd" | "exit" | "jeofetch" | "help" | "version")
}

pub fn is_executable(cmd: &str) -> bool {
    if Path::new(cmd).is_file() {
        return true;
    }
    let path_var = env::var_os("PATH").unwrap_or_default();
    for path in env::split_paths(&path_var) {
        let exe_path = path.join(cmd);
        if exe_path.is_file() {
            return true;
        }
        #[cfg(target_os = "windows")]
        {
            if path.join(format!("{}.exe", cmd)).is_file() {
                return true;
            }
        }
    }
    false
}
pub fn handle_builtin(args: &[String], state: &mut ShellState) -> Option<i32> {
    if args.is_empty() {
        return Some(0);
    }
    let cmd = &args[0];
    
    // Check if the command is a shortcut to go back: ".-1", "$PWD_BACK", "$PB"
    let is_back_cmd = cmd == ".-1" || cmd == "$PWD_BACK" || cmd == "$PB";
    let is_cd_back_cmd = cmd == "cd" && args.len() > 1 && (args[1] == ".-1" || args[1] == "$PWD_BACK" || args[1] == "$PB");

    if is_back_cmd || is_cd_back_cmd {
        if let Some(ref prev) = state.old_pwd {
            let prev_clone = prev.clone();
            let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            if let Err(e) = env::set_current_dir(&prev_clone) {
                eprintln!("cd: {}", e);
                return Some(1);
            }
            println!("{}", prev_clone.display());
            state.old_pwd = Some(current);
            return Some(0);
        } else {
            eprintln!("cd: nenhuma pasta anterior gravada.");
            return Some(1);
        }
    }

    // Auto-cd behavior: typing a directory directly moves into it
    if Path::new(cmd).is_dir() {
        let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if let Err(e) = env::set_current_dir(cmd) {
            eprintln!("cd: {}", e);
            return Some(1);
        }
        state.old_pwd = Some(current);
        return Some(0);
    }

    match cmd.as_str() {
        "cd" => {
            let target = if args.len() > 1 {
                Path::new(&args[1]).to_path_buf()
            } else {
                state.home_dir.clone()
            };
            let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            if let Err(e) = env::set_current_dir(&target) {
                eprintln!("cd: {}", e);
                Some(1)
            } else {
                state.old_pwd = Some(current);
                Some(0)
            }
        }
        "jeofetch" => {
            run_jeofetch();
            Some(0)
        }
        "exit" => {
            std::process::exit(0);
        }
        _ => None,
    }
}

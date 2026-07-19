use std::env;

fn find_in_path(cmd: &str) -> Option<String> {
    let path_var = env::var_os("PATH").unwrap_or_default();
    for path in env::split_paths(&path_var) {
        let exe_path = path.join(cmd);
        if exe_path.is_file() {
            return Some(exe_path.display().to_string());
        }
        #[cfg(target_os = "windows")]
        {
            let exe_path_win = path.join(format!("{}.exe", cmd));
            if exe_path_win.is_file() {
                return Some(exe_path_win.display().to_string());
            }
        }
    }
    None
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        println!("Uso: which <comando>");
        return;
    }
    let mut any_not_found = false;
    for cmd in &args {
        match find_in_path(cmd) {
            Some(path) => println!("{}", path),
            None => any_not_found = true,
        }
    }
    if any_not_found {
        std::process::exit(1);
    }
}
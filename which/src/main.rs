use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        println!("Uso: which <comando>");
        return;
    }
    let cmd = &args[0];
    let path_var = env::var_os("PATH").unwrap_or_default();
    let mut found = false;
    for path in env::split_paths(&path_var) {
        let exe_path = path.join(cmd);
        if exe_path.is_file() {
            println!("{}", exe_path.display());
            found = true;
            break;
        }
        #[cfg(target_os = "windows")]
        {
            let exe_path_win = path.join(format!("{}.exe", cmd));
            if exe_path_win.is_file() {
                println!("{}", exe_path_win.display());
                found = true;
                break;
            }
        }
    }
    if !found {
        std::process::exit(1);
    }
}
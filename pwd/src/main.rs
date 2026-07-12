use std::env;
use std::path::PathBuf;

fn print_usage() {
    eprintln!("Usage: pwd [-L | -P]");
    eprintln!("Print the name of the current working directory.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -L, --logical   use PWD from environment, even if it contains symlinks");
    eprintln!("  -P, --physical  avoid all symlinks");
    eprintln!("  -h, --help      display this help and exit");
    eprintln!("      --version   output version information and exit");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut mode = 'L'; // Default is logical (-L) for shell compatibility

    for arg in &args {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                return;
            }
            "--version" => {
                println!("pwd (JeffUtils) 1.0");
                return;
            }
            "-L" | "--logical" => {
                mode = 'L';
            }
            "-P" | "--physical" => {
                mode = 'P';
            }
            _ => {
                print_usage();
                std::process::exit(1);
            }
        }
    }

    let physical_path = match env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("pwd: error retrieving current directory: {}", e);
            std::process::exit(1);
        }
    };

    if mode == 'L' {
        if let Ok(pwd_env) = env::var("PWD") {
            let pwd_path = PathBuf::from(&pwd_env);
            if pwd_path.is_absolute() {
                if let Ok(canon_pwd) = pwd_path.canonicalize() {
                    if let Ok(canon_phys) = physical_path.canonicalize() {
                        if canon_pwd == canon_phys {
                            println!("{}", pwd_env);
                            return;
                        }
                    }
                }
            }
        }
    }

    // Physical mode (-P) or fallback
    if let Ok(canon) = physical_path.canonicalize() {
        // Strip UNC prefix on Windows if it gets added (e.g. \\?\)
        let mut path_str = canon.to_string_lossy().into_owned();
        if path_str.starts_with(r"\\?\") {
            path_str = path_str[4..].to_string();
        }
        println!("{}", path_str);
    } else {
        println!("{}", physical_path.display());
    }
}

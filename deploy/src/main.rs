use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

fn print_usage() {
    eprintln!("Usage: deploy [OPTION]... SOURCE DEST");
    eprintln!("       deploy [OPTION]... SOURCE... DIR");
    eprintln!("Copy SOURCE to DEST with permission/ownership setting.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -m MODE   set permission mode (octal, e.g. 755)");
    eprintln!("  -o OWNER  set owner (username)");
    eprintln!("  -g GROUP  set group (group name)");
    eprintln!("  -s        strip debug symbols (ignored)");
    eprintln!("  -b        make backup before overwriting");
    eprintln!("  -D        create all leading components of DEST first");
    eprintln!("  -h, --help       display this help and exit");
    eprintln!("  -v, --version    display version and exit");
}

fn parse_mode(s: &str) -> Result<u32, String> {
    let s = s.trim_start_matches('0');
    u32::from_str_radix(s, 8).map_err(|e| format!("invalid mode '{}': {}", s, e))
}

fn copy_file(src: &Path, dest: &Path, mode: Option<u32>, do_backup: bool, create_dirs: bool) -> Result<(), String> {
    if dest.exists() && do_backup {
        let backup = dest.with_extension(
            format!("{}.bak", dest.extension().unwrap_or_default().to_string_lossy()),
        );
        if dest.is_dir() {
            fs::remove_dir_all(&backup).map_err(|e| format!("backup: {}", e))?;
        } else {
            fs::copy(dest, &backup).map_err(|e| format!("backup: {}", e))?;
        }
    }

    if let Some(parent) = dest.parent() {
        if create_dirs {
            fs::create_dir_all(parent).map_err(|e| format!("create dirs: {}", e))?;
        }
    }

    if src.is_dir() {
        fs::create_dir_all(dest).map_err(|e| format!("mkdir: {}", e))?;
        for entry in fs::read_dir(src).map_err(|e| format!("read dir: {}", e))? {
            let entry = entry.map_err(|e| format!("read dir: {}", e))?;
            let file_name = entry.file_name();
            let new_dest = dest.join(&file_name);
            copy_file(&entry.path(), &new_dest, mode, do_backup, create_dirs)?;
        }
    } else {
        fs::copy(src, dest).map_err(|e| format!("copy: {}", e))?;
    }

    if let Some(m) = mode {
        fs::set_permissions(dest, fs::Permissions::from_mode(m))
            .map_err(|e| format!("chmod: {}", e))?;
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("deploy (JeffUtils) 1.0");
        return;
    }

    let mut mode: Option<u32> = None;
    let mut do_backup = false;
    let mut create_dirs = false;
    let mut sources: Vec<String> = Vec::new();
    let mut dest: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-m" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("deploy: option '-m' requires an argument");
                    std::process::exit(1);
                }
                mode = Some(parse_mode(&args[i]).unwrap_or_else(|e| {
                    eprintln!("deploy: {}", e);
                    std::process::exit(1);
                }));
            }
            "-o" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("deploy: option '-o' requires an argument");
                    std::process::exit(1);
                }
                // owner setting noted but not implemented on all platforms
            }
            "-g" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("deploy: option '-g' requires an argument");
                    std::process::exit(1);
                }
            }
            "-s" => {}
            "-b" => do_backup = true,
            "-D" => create_dirs = true,
            "--version" => {}
            "--help" => {}
            other if other.starts_with('-') && other.len() > 1 => {
                // Handle combined short flags
                for ch in other[1..].chars() {
                    match ch {
                        's' => {}
                        'b' => do_backup = true,
                        'D' => create_dirs = true,
                        _ => {
                            eprintln!("deploy: unknown option '-{}'", ch);
                            std::process::exit(1);
                        }
                    }
                }
            }
            _ => {
                sources.push(args[i].clone());
            }
        }
        i += 1;
    }

    if sources.len() < 1 {
        eprintln!("deploy: missing operand");
        eprintln!("Try 'deploy --help' for more information.");
        std::process::exit(1);
    }

    // Determine DEST: last arg is dest if multiple sources, or second arg if one source
    if sources.len() >= 2 {
        dest = Some(sources.pop().unwrap());
    }

    let dest = match dest {
        Some(d) => d,
        None => {
            eprintln!("deploy: missing destination operand");
            std::process::exit(1);
        }
    };

    let dest_path = PathBuf::from(&dest);

    if sources.len() > 1 && !dest_path.is_dir() {
        eprintln!("deploy: target '{}' is not a directory", dest);
        std::process::exit(1);
    }

    for src in &sources {
        let src_path = Path::new(src);
        if !src_path.exists() {
            eprintln!("deploy: cannot stat '{}': No such file or directory", src);
            std::process::exit(1);
        }

        let target = if sources.len() == 1 && dest_path.is_dir() {
            dest_path.join(src_path.file_name().unwrap_or_default())
        } else if sources.len() > 1 {
            dest_path.join(src_path.file_name().unwrap_or_default())
        } else {
            dest_path.clone()
        };

        if let Err(e) = copy_file(src_path, &target, mode, do_backup, create_dirs) {
            eprintln!("deploy: {}", e);
            std::process::exit(1);
        }
    }
}

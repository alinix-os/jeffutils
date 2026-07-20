use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn print_usage() {
    eprintln!("Usage: temppath [OPTION]...");
    eprintln!("Create a temporary file or directory safely.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -d               create a directory instead of a file");
    eprintln!("  -p DIR           use DIR as parent directory");
    eprintln!("  -u               dry run: just print the path, don't create anything");
    eprintln!("  --template TMPL  template with XXXXXX (replaced with random chars)");
    eprintln!("                   (default: /tmp/tmp.XXXXXX)");
    eprintln!("  -h, --help       display this help and exit");
    eprintln!("  -v, --version    display version and exit");
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("temppath", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("temppath (JeffUtils) 1.0");
        return;
    }

    let mut make_dir = false;
    let mut dry_run = false;
    let mut parent: Option<String> = None;
    let mut template: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" => make_dir = true,
            "-u" => dry_run = true,
            "-p" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("temppath: option '-p' requires an argument");
                    std::process::exit(1);
                }
                parent = Some(args[i].clone());
            }
            "--template" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("temppath: option '--template' requires an argument");
                    std::process::exit(1);
                }
                template = Some(args[i].clone());
            }
            other if other.starts_with("--template=") => {
                template = Some(other["--template=".len()..].to_string());
            }
            other if other.starts_with('-') && other.len() > 1 => {
                eprintln!("temppath: unknown option '{}'", other);
                std::process::exit(1);
            }
            _ => {
                eprintln!("temppath: unexpected argument '{}'", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let tmpl = template.unwrap_or_else(|| "/tmp/tmp.XXXXXX".to_string());

    let base_dir = parent
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut p = PathBuf::from(&tmpl);
            p.pop();
            if p.as_os_str().is_empty() {
                env::temp_dir()
            } else {
                p
            }
        });

    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let suffix: String = (0..6)
            .map(|_| {
                let idx = rng.gen_range(0..36);
                if idx < 10 {
                    (b'0' + idx) as char
                } else {
                    (b'a' + idx - 10) as char
                }
            })
            .collect();

        let tmpl_name = Path::new(&tmpl).file_name().unwrap_or_default().to_string_lossy();
        let name = tmpl_name.replace("XXXXXX", &suffix);
        let candidate = base_dir.join(&name);

        if !candidate.exists() {
            if dry_run {
                println!("{}", candidate.display());
                return;
            }
            if make_dir {
                match fs::create_dir(&candidate) {
                    Ok(()) => {
                        println!("{}", candidate.display());
                        return;
                    }
                    Err(_) => continue,
                }
            } else {
                match fs::File::create_new(&candidate) {
                    Ok(_) => {
                        println!("{}", candidate.display());
                        return;
                    }
                    Err(_) => continue,
                }
            }
        }
    }

    eprintln!("temppath: failed to create temporary path after 1000 attempts");
    std::process::exit(1);
}

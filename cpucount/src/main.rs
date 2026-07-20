use std::env;

const VERSION: &str = "0.1.0";

fn print_usage() {
    let name = env::args().next().unwrap_or_else(|| "cpucount".into());
    eprintln!("Usage: {name} [--all] [--ignore=N]");
    eprintln!("Print the number of available processors.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --all       print the total number of processors including offline");
    eprintln!("  --ignore=N  subtract N from the processor count");
    eprintln!("  -h, --help  show this help message");
    eprintln!("  -v, --version show version");
}

fn get_online_processors() -> i64 {
    unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) }
}

fn get_total_processors() -> i64 {
    unsafe { libc::sysconf(libc::_SC_NPROCESSORS_CONF) }
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("cpucount", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "-v" || arg == "--version" {
            println!("cpucount {VERSION}");
            return;
        }
    }

    let mut all = false;
    let mut ignore: i64 = 0;

    for arg in &args {
        if arg == "--all" {
            all = true;
        } else if let Some(val) = arg.strip_prefix("--ignore=") {
            ignore = val.parse().unwrap_or_else(|_| {
                eprintln!("cpucount: invalid number for --ignore: '{val}'");
                std::process::exit(1);
            });
        } else {
            eprintln!("cpucount: unknown option '{arg}'");
            std::process::exit(1);
        }
    }

    let count = if all {
        get_total_processors()
    } else {
        get_online_processors()
    };

    let result = (count - ignore).max(0);
    println!("{result}");
}

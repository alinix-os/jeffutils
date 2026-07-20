fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("umount", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        args.insert(1, "unmount".into());
    } else {
        args.push("unmount".into());
    }
    let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let status = std::process::Command::new("disk")
        .args(&args[1..])
        .status();

    match status {
        Ok(s) if s.success() => {},
        Ok(_) => std::process::exit(1),
        Err(e) => {
            eprintln!("Error: could not execute disk: {}", e);
            std::process::exit(1);
        }
    }
}

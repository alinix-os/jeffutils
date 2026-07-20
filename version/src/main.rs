fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("version", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    println!("JeffUtils v{}", env!("CARGO_PKG_VERSION"));
}
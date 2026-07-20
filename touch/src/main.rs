fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("touch", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    create::run();
}

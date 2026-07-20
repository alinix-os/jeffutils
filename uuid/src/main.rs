use uuid::Uuid;

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("uuid", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let id = Uuid::new_v4();
    println!("{}", id);
}
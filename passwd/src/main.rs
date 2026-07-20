fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("passwd", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    println!("passwd: Alteração de senha não suportada nesta plataforma ou requer API do kernel.");
}
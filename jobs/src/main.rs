fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("jobs", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    println!("jobs: Este é um comando interno do shell para gerenciar tarefas em segundo plano.");
}
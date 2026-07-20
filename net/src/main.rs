use sysinfo::Networks;

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("net", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let networks = Networks::new_with_refreshed_list();
    println!("=== Interfaces de Rede ===");
    for (interface_name, network) in &networks {
        println!("{}:", interface_name);
        println!("  Recebido: {} B", network.received());
        println!("  Transmitido: {} B", network.transmitted());
    }
}
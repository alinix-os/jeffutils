use sysinfo::Networks;

fn main() {
    let networks = Networks::new_with_refreshed_list();
    println!("=== Interfaces de Rede ===");
    for (interface_name, network) in &networks {
        println!("{}:", interface_name);
        println!("  Recebido: {} B", network.received());
        println!("  Transmitido: {} B", network.transmitted());
    }
}
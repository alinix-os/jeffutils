fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        println!("nice: Nenhum comando especificado.");
        println!("Uso: nice <prioridade> <comando> [args...]");
        return;
    }
    println!("nice: Configuração de prioridade não suportada nesta plataforma ou requer privilégios elevados.");
}
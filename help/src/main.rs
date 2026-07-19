use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        println!("=== JeffUtils Help ===");
        println!("Uso: help <comando>");
        println!("Exibe a documentação de um comando do sistema.");
        return;
    }
    let cmd = &args[0];
    let help_dir = env::var("JEFFUTILS_HELP_DIR").unwrap_or_else(|_| "/Shared/help".to_string());
    let help_path = Path::new(&help_dir).join(cmd);
    if help_path.exists() {
        match fs::read_to_string(&help_path) {
            Ok(content) => println!("{}", content),
            Err(e) => eprintln!("Erro ao ler ajuda para {}: {}", cmd, e),
        }
    } else {
        println!("Nenhuma documentação detalhada encontrada para '{}'.", cmd);
        println!("Tente usar: {} --help", cmd);
    }
}
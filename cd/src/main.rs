fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        println!("cd: Nenhum diretório especificado.");
    } else {
        let target = &args[0];
        println!("cd: Para mudar de diretório, use o comando interno do shell.");
        println!("Caminho sugerido: {}", target);
    }
}
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Uso: stat <arquivo/diretório>");
        std::process::exit(1);
    }
    let path_str = &args[0];
    let path = Path::new(path_str);
    if !path.exists() {
        eprintln!("stat: '{}' não encontrado.", path_str);
        std::process::exit(1);
    }
    match fs::metadata(path) {
        Ok(meta) => {
            println!("  Arquivo: {}", path_str);
            println!("  Tamanho: {} bytes", meta.len());
            println!("  Tipo: {}", if meta.is_dir() { "Diretório" } else if meta.is_file() { "Arquivo Regular" } else { "Link/Outro" });
            if let Ok(modified) = meta.modified() {
                println!("Modificado: {:?}", modified);
            }
            if let Ok(accessed) = meta.accessed() {
                println!(" Acessado: {:?}", accessed);
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                println!("     Perms: {:o}", meta.permissions().readonly() as u32);
                println!("       UID: {}", meta.uid());
                println!("       GID: {}", meta.gid());
            }
        }
        Err(e) => eprintln!("Erro ao obter metadados: {}", e),
    }
}
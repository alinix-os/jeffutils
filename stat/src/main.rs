use std::env;
use std::fs;
use std::path::Path;

fn print_usage() {
    eprintln!("Uso: stat [OPÇÃO] <arquivo/diretório>");
    eprintln!("Exibe o status de um arquivo ou diretório.");
    eprintln!();
    eprintln!("Opções:");
    eprintln!("  -h, --help      exibe esta ajuda e sai");
    eprintln!("      --version   exibe a versão e sai");
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("stat", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    // Check help and version
    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "--version" {
            println!("stat (JeffUtils) 1.0");
            return;
        }
    }

    if args.is_empty() {
        print_usage();
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
            println!("  Arquivo : {}", path_str);
            println!("  Tamanho : {} bytes", meta.len());
            println!("  Tipo    : {}", if meta.is_dir() { "Diretório" } else if meta.is_file() { "Arquivo Regular" } else { "Link/Outro" });
            if let Ok(modified) = meta.modified() {
                println!("Modificado: {:?}", modified);
            }
            if let Ok(accessed) = meta.accessed() {
                println!(" Acessado : {:?}", accessed);
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::{MetadataExt, PermissionsExt};
                let mode = meta.permissions().mode();
                println!("     Perms: {:o} ({})", mode & 0o777, format_mode(mode));
                println!("       UID: {}", meta.uid());
                println!("       GID: {}", meta.gid());
            }
        }
        Err(e) => eprintln!("Erro ao obter metadados: {}", e),
    }
}

#[cfg(unix)]
fn format_mode(mode: u32) -> String {
    let r  = if mode & 0o400 != 0 { "r" } else { "-" };
    let w  = if mode & 0o200 != 0 { "w" } else { "-" };
    let x  = if mode & 0o100 != 0 {
        if mode & 0o4000 != 0 { "s" } else { "x" }
    } else {
        if mode & 0o4000 != 0 { "S" } else { "-" }
    };
    let r2 = if mode & 0o040 != 0 { "r" } else { "-" };
    let w2 = if mode & 0o020 != 0 { "w" } else { "-" };
    let x2 = if mode & 0o010 != 0 {
        if mode & 0o2000 != 0 { "s" } else { "x" }
    } else {
        if mode & 0o2000 != 0 { "S" } else { "-" }
    };
    let r3 = if mode & 0o004 != 0 { "r" } else { "-" };
    let w3 = if mode & 0o002 != 0 { "w" } else { "-" };
    let x3 = if mode & 0o001 != 0 {
        if mode & 0o1000 != 0 { "t" } else { "x" }
    } else {
        if mode & 0o1000 != 0 { "T" } else { "-" }
    };
    format!("{}{}{}{}{}{}{}{}{}", r, w, x, r2, w2, x2, r3, w3, x3)
}

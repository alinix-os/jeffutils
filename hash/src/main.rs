use std::env;
use std::fs::File;
use std::io::Read;
use sha2::{Sha256, Digest};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    for arg in &args {
        if arg == "-h" || arg == "--help" {
            println!("Usage: hash <file>...");
            println!("Compute SHA-256 hash of each file.");
            return;
        }
        if arg == "--version" {
            println!("hash (JeffUtils) 1.0");
            return;
        }
    }
    if args.is_empty() {
        println!("Uso: hash <arquivo>");
        return;
    }
    let mut any_error = false;
    for path in &args {
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Erro ao abrir arquivo: {}", e);
                any_error = true;
                continue;
            }
        };
        let mut hasher = Sha256::new();
        let mut buffer = [0; 4096];
        loop {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(e) => {
                    eprintln!("Erro ao ler arquivo: {}", e);
                    any_error = true;
                    break;
                }
            }
        }
        let result = hasher.finalize();
        println!("{:x}", result);
    }
    if any_error {
        std::process::exit(1);
    }
    return;
}
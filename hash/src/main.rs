use std::env;
use std::fs::File;
use std::io::Read;
use sha2::{Sha256, Digest};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        println!("Uso: hash <arquivo>");
        return;
    }
    let path = &args[0];
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Erro ao abrir arquivo: {}", e);
            return;
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
                return;
            }
        }
    }
    let result = hasher.finalize();
    println!("{:x}", result);
}
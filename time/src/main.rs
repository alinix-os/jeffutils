use std::env;
use std::process::Command;
use std::time::Instant;

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("time", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();
    for arg in &args {
        if arg == "-h" || arg == "--help" {
            println!("Usage: time <command> [args...]");
            println!("Measure the execution time of a command.");
            return;
        }
        if arg == "--version" {
            println!("time (JeffUtils) 1.0");
            return;
        }
    }
    if args.is_empty() {
        println!("Uso: time <comando> [args...]");
        return;
    }
    let cmd = &args[0];
    let cmd_args = &args[1..];
    let start = Instant::now();
    let status = Command::new(cmd)
        .args(cmd_args)
        .status();
    let duration = start.elapsed();
    match status {
        Ok(s) => {
            eprintln!("\nTempo de execução: {:?}", duration);
            if !s.success() {
                std::process::exit(s.code().unwrap_or(1));
            }
        }
        Err(e) => {
            eprintln!("Erro ao executar comando: {}", e);
            std::process::exit(1);
        }
    }
}
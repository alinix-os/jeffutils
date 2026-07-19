use std::env;
use std::net::{ToSocketAddrs, TcpStream};
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        println!("Uso: ping <host> [porta]");
        return;
    }
    let host = &args[0];
    let port = args.get(1).map(|s| s.as_str()).unwrap_or("80");
    let addr_str = format!("{}:{}", host, port);
    println!("Pingando {} via TCP na porta {}...", host, port);
    
    let mut failures = 0;
    match addr_str.to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                for i in 1..=4 {
                    let start = Instant::now();
                    match TcpStream::connect_timeout(&addr, Duration::from_secs(2)) {
                        Ok(_) => {
                            let duration = start.elapsed();
                            println!("Resposta de {}: sequencia={} tempo={:?}", addr, i, duration);
                        }
                        Err(e) => {
                            println!("Falha ao conectar a {}: sequencia={} erro={}", addr, i, e);
                            failures += 1;
                        }
                    }
                    if i < 4 {
                        std::thread::sleep(Duration::from_secs(1));
                    }
                }
            } else {
                eprintln!("Não foi possível resolver o host: {}", host);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Erro de resolução DNS: {}", e);
            std::process::exit(1);
        }
    }
    if failures == 4 {
        std::process::exit(1);
    }
}
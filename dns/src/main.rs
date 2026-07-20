use std::env;
use std::fs;
use std::net::ToSocketAddrs;
use std::path::Path;

fn get_system_dns() -> Vec<String> {
    let mut dns_servers = Vec::new();
    #[cfg(unix)]
    {
        let resolv_path = "/etc/resolv.conf";
        if Path::new(resolv_path).exists() {
            if let Ok(content) = fs::read_to_string(resolv_path) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && parts[0] == "nameserver" {
                        dns_servers.push(parts[1].to_string());
                    }
                }
            }
        }
    }
    #[cfg(windows)]
    {
        dns_servers.push("8.8.8.8".to_string());
        dns_servers.push("1.1.1.1".to_string());
    }
    dns_servers
}

fn validate_dns_arg(arg: &str) -> Result<(), String> {
    if arg.is_empty() {
        return Err("DNS server argument is empty".into());
    }
    if arg.contains(char::is_whitespace) {
        return Err(format!("DNS server argument contains whitespace: '{}'", arg));
    }
    Ok(())
}

fn set_system_dns(d1: &str, d2: Option<&str>) -> Result<(), String> {
    validate_dns_arg(d1)?;
    if let Some(sec) = d2 {
        validate_dns_arg(sec)?;
    }
    #[cfg(unix)]
    {
        let resolv_path = "/etc/resolv.conf";
        let existing = fs::read_to_string(resolv_path).unwrap_or_default();
        let mut new_nameservers = vec![format!("nameserver {}", d1)];
        if let Some(sec) = d2 {
            new_nameservers.push(format!("nameserver {}", sec));
        }
        let mut lines: Vec<String> = existing.lines().map(String::from).collect();
        let mut replaced = 0;
        for line in &mut lines {
            let trimmed = line.trim_start();
            if trimmed.starts_with("nameserver") {
                if replaced < new_nameservers.len() {
                    *line = new_nameservers[replaced].clone();
                    replaced += 1;
                }
            }
        }
        if replaced < new_nameservers.len() {
            for extra in &new_nameservers[replaced..] {
                lines.push(extra.clone());
            }
        }
        let content: String = lines.join("\n");
        let content = if content.ends_with('\n') { content } else { format!("{}\n", content) };
        fs::write(resolv_path, content).map_err(|e| format!("Erro ao escrever em {}: {}", resolv_path, e))
    }
    #[cfg(windows)]
    {
        println!("Aviso: Configuração DNS persistida apenas de forma mock no Windows.");
        Ok(())
    }
}

fn resolve_dns(host: &str) {
    let query = format!("{}:0", host);
    println!("Resolvendo {}...", host);
    match query.to_socket_addrs() {
        Ok(addrs) => {
            for addr in addrs {
                println!("  IP: {}", addr.ip());
            }
        }
        Err(e) => eprintln!("Erro ao resolver DNS: {}", e),
    }
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("dns", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        let servers = get_system_dns();
        println!("=== Servidores DNS Atuais ===");
        if servers.is_empty() {
            println!("Nenhum servidor DNS detectado.");
        } else {
            for (i, s) in servers.iter().enumerate() {
                println!("  DNS {}: {}", i + 1, s);
            }
        }
        println!("\nUso:");
        println!("  dns <hostname>             - Resolve um nome de domínio");
        println!("  dns check <hostname>       - Consulta a resolução de um domínio");
        println!("  dns --set <d1> [d2]        - Configura servidores DNS primário e secundário");
        return;
    }

    if args[0] == "--set" {
        if args.len() < 2 {
            eprintln!("Erro: especifique pelo menos o DNS primário.");
            eprintln!("Uso: dns --set <d1> [d2]");
            std::process::exit(1);
        }
        let d1 = &args[1];
        let d2 = args.get(2).map(|s| s.as_str());

        match set_system_dns(d1, d2) {
            Ok(_) => {
                println!("Servidores DNS atualizados com sucesso!");
                println!("  Primário: {}", d1);
                if let Some(sec) = d2 {
                    println!("  Secundário: {}", sec);
                }
            }
            Err(e) => {
                eprintln!("Erro ao atualizar DNS: {}", e);
                eprintln!("Nota: Esta operação pode exigir privilégios de administrador (sudo).");
                std::process::exit(1);
            }
        }
        return;
    }

    if args[0] == "check" {
        if args.len() < 2 {
            eprintln!("Erro: especifique o hostname para verificação.");
            eprintln!("Uso: dns check <hostname>");
            std::process::exit(1);
        }
        resolve_dns(&args[1]);
        return;
    }

    resolve_dns(&args[0]);
}
use std::env;
use std::process::Command;

fn print_usage() {
    eprintln!("Uso: nice [OPÇÃO] [COMANDO [ARG]...]");
    eprintln!("Executa um COMANDO com a prioridade de escalonamento (nice) ajustada.");
    eprintln!("Sem COMANDO, exibe a prioridade atual.");
    eprintln!();
    eprintln!("Opções:");
    eprintln!("  -n, --adjustment=N   adiciona N à prioridade (padrão 10)");
    eprintln!("  -h, --help           exibe esta ajuda e sai");
    eprintln!("      --version        exibe a versão e sai");
}

#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "ios"))]
unsafe fn clear_errno() { unsafe { *libc::__error() = 0; } }
#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "ios"))]
unsafe fn get_errno() -> i32 { unsafe { *libc::__error() } }

#[cfg(any(target_os = "linux", target_os = "android"))]
unsafe fn clear_errno() { unsafe { *libc::__errno_location() = 0; } }
#[cfg(any(target_os = "linux", target_os = "android"))]
unsafe fn get_errno() -> i32 { unsafe { *libc::__errno_location() } }

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // Check help and version
    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "--version" {
            println!("nice (JeffUtils) 1.0");
            return;
        }
    }

    let mut adjustment = 10;
    let mut command_start = 0;

    if !args.is_empty() {
        if args[0] == "-n" {
            if args.len() < 2 {
                eprintln!("nice: a opção '-n' requer um argumento");
                std::process::exit(1);
            }
            adjustment = match args[1].parse::<i32>() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("nice: argumento inválido: '{}'", args[1]);
                    std::process::exit(1);
                }
            };
            command_start = 2;
        } else if args[0].starts_with("-n") && args[0].len() > 2 {
            let val = &args[0][2..];
            adjustment = match val.parse::<i32>() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("nice: argumento inválido: '{}'", val);
                    std::process::exit(1);
                }
            };
            command_start = 1;
        } else if args[0].starts_with("+") {
            let val = &args[0][1..];
            adjustment = match val.parse::<i32>() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("nice: argumento inválido: '{}'", val);
                    std::process::exit(1);
                }
            };
            command_start = 1;
        } else if args[0].starts_with("--adjustment=") {
            let val = args[0].trim_start_matches("--adjustment=");
            adjustment = match val.parse::<i32>() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("nice: argumento inválido: '{}'", val);
                    std::process::exit(1);
                }
            };
            command_start = 1;
        } else if args[0].starts_with("-") {
            // Try to parse as integer directly (e.g. -5)
            if let Ok(n) = args[0].parse::<i32>() {
                adjustment = n;
                command_start = 1;
            }
        }
    }

    // If no command is specified, print current priority
    if command_start >= args.len() {
        #[cfg(unix)]
        {
            unsafe {
                clear_errno();
                let prio = libc::getpriority(libc::PRIO_PROCESS, 0);
                let errno = get_errno();
                if prio == -1 && errno != 0 {
                    eprintln!("nice: não foi possível obter a prioridade atual: {}", std::io::Error::last_os_error());
                    std::process::exit(1);
                } else {
                    println!("{}", prio);
                    return;
                }
            }
        }
        #[cfg(not(unix))]
        {
            println!("0");
            return;
        }
    }

    let cmd_name = &args[command_start];
    let cmd_args = &args[command_start + 1..];

    // Set nice priority if on Unix
    #[cfg(unix)]
    {
        unsafe {
            clear_errno();
            let current = libc::getpriority(libc::PRIO_PROCESS, 0);
            let errno = get_errno();
            if !(current == -1 && errno != 0) {
                let target = (current + adjustment).max(-20).min(19);
                if libc::setpriority(libc::PRIO_PROCESS, 0, target) == -1 {
                    let err = std::io::Error::last_os_error();
                    eprintln!("nice: não foi possível definir a prioridade para {}: {}", target, err);
                    // Standard nice still runs the command even if setting priority fails (e.g. permission denied)
                }
            }
        }
    }

    // Run the command
    let mut child = match Command::new(cmd_name).args(cmd_args).spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("nice: falha ao executar '{}': {}", cmd_name, e);
            std::process::exit(127);
        }
    };

    let status = match child.wait() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("nice: erro ao aguardar processo filho: {}", e);
            std::process::exit(1);
        }
    };

    std::process::exit(status.code().unwrap_or(0));
}
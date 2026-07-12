use crate::shell::ShellState;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn run_jeofetch() {
    let status = Command::new("jeofetch")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    if let Err(e) = status {
        eprintln!("jsh: jeofetch: {}", e);
    }
}

pub fn is_builtin(cmd: &str) -> bool {
    matches!(
        cmd,
        "cd" | "exit"
            | "jeofetch"
            | "help"
            | "version"
            | "export"
            | "unset"
            | "set"
            | "alias"
            | "unalias"
            | "source"
            | "."
            | "true"
            | "false"
            | ":"
    )
}

pub fn is_executable(cmd: &str) -> bool {
    if Path::new(cmd).is_file() {
        return true;
    }
    let path_var = env::var_os("PATH").unwrap_or_default();
    for path in env::split_paths(&path_var) {
        let exe_path = path.join(cmd);
        if exe_path.is_file() {
            return true;
        }
        #[cfg(target_os = "windows")]
        {
            if path.join(format!("{}.exe", cmd)).is_file() {
                return true;
            }
        }
    }
    false
}

fn print_help() {
    println!(
        "\
jsh — shell interativo

Builtins:
  cd [dir]           Muda de diretório (sem args: vai para $HOME)
  export NAME=valor   Define e exporta uma variável de ambiente
  export NAME         Exporta uma variável de shell já existente
  unset NAME          Remove uma variável de shell/ambiente
  set                 Lista variáveis de shell e de ambiente
  alias nome=valor    Define um alias
  unalias nome        Remove um alias
  source arquivo | .  Executa um script no shell atual
  true / false / :    Comandos no-op de status 0/1
  exit                Sai do jsh

Sintaxe suportada: pipes (|), redirecionamentos (>, >>, <, <<, <<<, 2>, &>),
listas de comandos (;, &&, ||), aspas simples/duplas, escapes (\\),
substituição de comando $(...) / `...`, variáveis de shell e $?, $$, $0,
globbing (*, ?, [...]), histórico !! / !n / !prefixo."
    );
}

pub fn handle_builtin(args: &[String], state: &mut ShellState) -> Option<i32> {
    if args.is_empty() {
        return Some(0);
    }
    let cmd = &args[0];

    // Check if the command is a shortcut to go back: ".-1", "$PWD_BACK", "$PB"
    let is_back_cmd = cmd == ".-1" || cmd == "$PWD_BACK" || cmd == "$PB";
    let is_cd_back_cmd = cmd == "cd" && args.len() > 1 && (args[1] == ".-1" || args[1] == "$PWD_BACK" || args[1] == "$PB");

    if is_back_cmd || is_cd_back_cmd {
        if let Some(ref prev) = state.old_pwd {
            let prev_clone = prev.clone();
            let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            if let Err(e) = env::set_current_dir(&prev_clone) {
                eprintln!("cd: {}", e);
                return Some(1);
            }
            println!("{}", prev_clone.display());
            state.old_pwd = Some(current);
            return Some(0);
        } else {
            eprintln!("cd: nenhuma pasta anterior gravada.");
            return Some(1);
        }
    }

    // Auto-cd behavior: typing a directory directly moves into it. Only
    // kicks in when `cmd` isn't otherwise runnable (a real PATH executable
    // or a user function), so a local dir that happens to share a name
    // with a real command (e.g. a `./pwd/` subfolder) doesn't shadow it.
    if args.len() == 1
        && Path::new(cmd).is_dir()
        && !is_executable(cmd)
        && !state.functions.contains_key(cmd)
    {
        let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if let Err(e) = env::set_current_dir(cmd) {
            eprintln!("cd: {}", e);
            return Some(1);
        }
        state.old_pwd = Some(current);
        return Some(0);
    }

    match cmd.as_str() {
        "cd" => {
            let target = if args.len() > 1 {
                Path::new(&args[1]).to_path_buf()
            } else {
                state.home_dir.clone()
            };
            let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            if let Err(e) = env::set_current_dir(&target) {
                if !state.quiet_errors {
                    eprintln!("cd: {}", e);
                }
                Some(1)
            } else {
                state.old_pwd = Some(current);
                Some(0)
            }
        }
        "jeofetch" => {
            run_jeofetch();
            Some(0)
        }
        "help" => {
            print_help();
            Some(0)
        }
        "version" => {
            println!("jsh {}", env!("CARGO_PKG_VERSION"));
            Some(0)
        }
        "true" | ":" => Some(0),
        "false" => Some(1),
        "export" => {
            for arg in &args[1..] {
                if let Some(eq) = arg.find('=') {
                    let (name, value) = (&arg[..eq], &arg[eq + 1..]);
                    state.export_var(name, Some(value));
                } else {
                    state.export_var(arg, None);
                }
            }
            Some(0)
        }
        "unset" => {
            for arg in &args[1..] {
                state.unset_var(arg);
            }
            Some(0)
        }
        "set" => {
            let mut names: Vec<&String> = state.shell_vars.keys().collect();
            names.sort();
            for name in names {
                println!("{}={}", name, state.shell_vars[name]);
            }
            Some(0)
        }
        "alias" => {
            if args.len() == 1 {
                let map = state.aliases.lock().unwrap();
                let mut names: Vec<&String> = map.keys().collect();
                names.sort();
                for name in names {
                    println!("alias {}='{}'", name, map[name]);
                }
            } else {
                let mut map = state.aliases.lock().unwrap();
                for arg in &args[1..] {
                    if let Some(eq) = arg.find('=') {
                        let name = &arg[..eq];
                        let value = arg[eq + 1..].trim_matches('"').trim_matches('\'');
                        map.insert(name.to_string(), value.to_string());
                    } else if let Some(v) = map.get(arg) {
                        println!("alias {}='{}'", arg, v);
                    }
                }
            }
            Some(0)
        }
        "unalias" => {
            let mut map = state.aliases.lock().unwrap();
            for arg in &args[1..] {
                map.remove(arg);
            }
            Some(0)
        }
        "source" | "." => {
            if args.len() < 2 {
                eprintln!("{}: nome de arquivo esperado", cmd);
                return Some(1);
            }
            match fs::read_to_string(&args[1]) {
                Ok(content) => {
                    if ShellState::looks_like_bash(&content) {
                        state.bash_sourced_files.push(PathBuf::from(&args[1]));
                    } else {
                        state.run_script_text(&content);
                    }
                    Some(state.last_exit_status)
                }
                Err(e) => {
                    if !state.quiet_errors {
                        eprintln!("{}: {}: {}", cmd, args[1], e);
                    }
                    Some(1)
                }
            }
        }
        "exit" => {
            let code = args
                .get(1)
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            std::process::exit(code);
        }
        _ => None,
    }
}

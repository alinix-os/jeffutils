use std::process::Command;

pub mod pipeline;

pub fn execute_command(args: &[String]) -> i32 {
    if args.is_empty() {
        return 0;
    }
    let cmd = &args[0];
    let mut child = match Command::new(cmd).args(&args[1..]).spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("jsh: {}: {}", cmd, e);
            return 127;
        }
    };
    match child.wait() {
        Ok(status) => status.code().unwrap_or(0),
        Err(e) => {
            eprintln!("jsh: erro ao aguardar processo: {}", e);
            1
        }
    }
}

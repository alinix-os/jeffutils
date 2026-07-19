use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};
use rustyline::history::DefaultHistory;

fn main() {
    let mut rl = Editor::<(), DefaultHistory>::new().unwrap();
    rl.add_history_entry("echo 1");
    rl.add_history_entry("cd /tmp");
    rl.add_history_entry("echo 2");
    
    rl.bind_sequence(
        rustyline::KeyEvent(rustyline::KeyCode::Up, rustyline::Modifiers::empty()),
        rustyline::Cmd::HistorySearchBackward,
    );
    
    match rl.readline("> ") {
        Ok(line) => println!("Line: {}", line),
        Err(e) => println!("Error: {:?}", e),
    }
}

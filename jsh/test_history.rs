use rustyline::{Config, Editor};
fn main() {
    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new().unwrap();
    rl.add_history_entry("hello").unwrap();
    rl.add_history_entry("world").unwrap();
    
    // Test if HistorySearchBackward works with empty prefix
}

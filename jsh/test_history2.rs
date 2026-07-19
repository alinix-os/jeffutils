use rustyline::{Editor, Config};
use rustyline::history::DefaultHistory;
fn main() {
    let mut rl = Editor::<(), DefaultHistory>::new().unwrap();
    rl.bind_sequence(
        rustyline::KeyEvent(rustyline::KeyCode::Up, rustyline::Modifiers::empty()),
        rustyline::Cmd::HistorySearchBackward,
    );
    // Let's just print to verify it compiles.
    println!("Compiled!");
}

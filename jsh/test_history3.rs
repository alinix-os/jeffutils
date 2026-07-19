use rustyline::{Config, Editor, Cmd, Event, RepeatCount};
use rustyline::history::DefaultHistory;
use rustyline::binding::{ConditionalEventHandler, EventContext, EventHandler};

struct UpArrowHandler;
impl ConditionalEventHandler for UpArrowHandler {
    fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
        if ctx.line().is_empty() {
            Some(Cmd::PreviousHistory)
        } else {
            Some(Cmd::HistorySearchBackward)
        }
    }
}

fn main() {
    let mut rl = Editor::<(), DefaultHistory>::new().unwrap();
    rl.bind_sequence(
        rustyline::KeyEvent(rustyline::KeyCode::Up, rustyline::Modifiers::empty()),
        EventHandler::Conditional(Box::new(UpArrowHandler)),
    );
}

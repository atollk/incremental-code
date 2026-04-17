mod backend;
mod basic_terminal_app;
pub mod demos;

use crate::basic_terminal_app::BasicTerminalApp;

pub fn main() {
    let app = BasicTerminalApp::<demos::CodeEditorDemo>::new();
    // let app = BasicTerminalApp::<demos::CounterDemo>::new();
    // let app = BasicTerminalApp::<BeamtermDemo>::new();
    app.run().unwrap();
}

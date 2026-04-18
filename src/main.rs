mod backend;
mod basic_terminal_app;
pub mod demos;
pub mod blinking_cursor;
pub mod code_editor;

use crate::basic_terminal_app::BasicTerminalApp;

pub fn main() {
    let app = BasicTerminalApp::<demos::CodeEditorDemo>::new();
    // let app = BasicTerminalApp::<demos::CounterDemo>::new();
    // let app = BasicTerminalApp::<BeamtermDemo>::new();
    app.run().unwrap();
}

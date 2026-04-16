mod backend;
mod basic_terminal_app;
mod beamterm_demo;
mod counter_demo;

use crate::basic_terminal_app::BasicTerminalApp;
use crate::beamterm_demo::BeamtermDemo;
use crate::counter_demo::CounterDemo;

pub fn main() {
    // let app = BasicTerminalApp::<CounterDemo>::new();
    let app = BasicTerminalApp::<BeamtermDemo>::new();
    app.run().unwrap();
}

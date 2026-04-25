mod backend;
mod basic_terminal_app;
pub mod demos;
pub mod game_scenes;
pub mod game_state;
pub mod widgets;

use crate::basic_terminal_app::BasicTerminalApp;
use crate::game_scenes::base::SceneGame;
use crate::game_scenes::home_terminal::HomeTerminalScene;

pub fn main() {
    // let app = BasicTerminalApp::new(demos::CodeEditorDemo::default());
    // let app = BasicTerminalApp::<demos::CounterDemo>::new();
    // let app = BasicTerminalApp::<BeamtermDemo>::new();
    let app = BasicTerminalApp::new(SceneGame::new(Box::new(HomeTerminalScene::new())));
    app.run().unwrap();
}

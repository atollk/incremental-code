#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![feature(impl_trait_in_bindings)]

mod backend;
mod basic_terminal_app;
pub mod demos;
pub mod game_scenes;
mod game_state;
pub mod widgets;

use crate::backend::audio::AUDIO_BACKEND;
use crate::backend::with_backend;
use crate::basic_terminal_app::BasicTerminalApp;
use crate::game_scenes::base::SceneGame;
use crate::game_scenes::home_terminal::HomeTerminalScene;
use std::ops::{Deref, DerefMut};

pub fn main() {
    with_backend(|backend| backend.init_logging().unwrap());
    {
        AUDIO_BACKEND.lock().unwrap().deref_mut().play().unwrap();
    }
    // let app = BasicTerminalApp::new(demos::CodeEditorDemo::default());
    // let app = BasicTerminalApp::<demos::CounterDemo>::new();
    // let app = BasicTerminalApp::<BeamtermDemo>::new();
    let app = BasicTerminalApp::new(SceneGame::new(Box::new(HomeTerminalScene::new())));
    app.run().unwrap();
}

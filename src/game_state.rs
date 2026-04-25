use language::CompiledProgram;
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, Mutex};

pub fn with_game_state<T>(f: impl Fn(&mut GameState) -> T) -> T {
    let mut lock = GLOBAL_GAME_STATE.lock().unwrap();
    f(&mut lock)
}

static GLOBAL_GAME_STATE: LazyLock<Mutex<GameState>> =
    LazyLock::new(|| Mutex::new(GameState::new()));

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub program_code: String,
    pub compiled_program: Option<CompiledProgram>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            program_code: String::new(),
            compiled_program: None,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState::new()
    }
}

use crate::game_state::Resources;
use crate::game_state::upgrades::Upgrades;
use language::CompiledProgram;
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, Mutex};

pub fn with_game_state<T>(f: impl Fn(&mut GameState) -> T) -> T {
    let mut lock = GLOBAL_GAME_STATE.lock().unwrap();
    f(&mut lock)
}

static GLOBAL_GAME_STATE: LazyLock<Mutex<GameState>> =
    LazyLock::new(|| Mutex::new(GameState::default()));

#[derive(Serialize, Deserialize)]
pub struct GameState {
    // Program
    pub program_code: String,
    pub compiled_program: Option<CompiledProgram>,
    // Resources
    pub current_resources: Resources,
    pub carryover_resources: Resources,
    // Upgrades
    pub upgrades: Upgrades,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            program_code: "def foo():\n  return 1;\nend\n\nfoo();".to_string(),
            compiled_program: None,
            current_resources: Resources::default(),
            carryover_resources: Resources::default(),
            upgrades: Upgrades::default(),
        }
    }
}

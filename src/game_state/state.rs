use crate::game_state::Resources;
use crate::game_state::upgrades::Upgrades;
use anyhow::{anyhow, bail};
use language::CompiledProgram;
use serde::{Deserialize, Serialize};
use std::ops::Add;
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
            current_resources: Resources::from_bronze(999.),
            carryover_resources: Resources::default(),
            upgrades: Upgrades::default(),
        }
    }
}

impl GameState {
    pub fn total_resources(&self) -> Resources {
        &self.current_resources + &self.carryover_resources
    }

    pub fn take_resources(&mut self, resources: &Resources) -> anyhow::Result<()> {
        // Backup resources in case of error.
        let carryover_resources_backup = self.carryover_resources.clone();
        let current_resources_backup = self.current_resources.clone();

        // Subtract from carryover first
        let cost_left = resources.saturating_sub(&self.carryover_resources);
        self.carryover_resources = self.carryover_resources.saturating_sub(resources);

        // Then subtract the leftovers from non-carryover
        let final_cost_left = cost_left.saturating_sub(&self.current_resources);
        self.current_resources = self.current_resources.saturating_sub(&cost_left);

        // Verify that everything could be subtracted
        if final_cost_left == Resources::default() {
            Ok(())
        } else {
            // Set back resources and return error result.
            self.current_resources = current_resources_backup;
            self.carryover_resources = carryover_resources_backup;
            bail!(
                "Could not take {:?} resources from available {:?} + {:?}.",
                resources,
                self.current_resources,
                self.carryover_resources
            );
        }
    }

    pub fn give_carryover_resources(&mut self, resources: &Resources) {
        self.carryover_resources = &self.carryover_resources + resources;
    }
}

use crate::game_state::upgrades::Upgrades;
use crate::game_state::{CompiledProgram, Resources};
use crate::global_variable;
use anyhow::bail;
use serde::{Deserialize, Serialize};

global_variable!(game_state, GameState);

/// Persistent game state stored in a global singleton.
///
/// Access it exclusively via [`with_game_state`].
#[derive(Serialize, Deserialize)]
pub struct GameState {
    // Program
    pub program_code: String,
    pub compiled_program: Option<Result<CompiledProgram, (String, Vec<u64>)>>,
    // Resources
    pub current_resources: Resources,
    pub carryover_resources: Resources,
    // Upgrades
    pub upgrades: Upgrades,
}

impl Default for GameState {
    fn default() -> Self {
        let start_code = r#""#;
        let start_resources = Resources::new(1e9, 1e9, 1e9, 1e9, 1e9);
        GameState {
            program_code: start_code.to_string(),
            compiled_program: None,
            current_resources: start_resources,
            carryover_resources: Resources::default(),
            upgrades: Upgrades::default(),
        }
    }
}

impl GameState {
    /// Returns the sum of current and carryover resources.
    pub fn total_resources(&self) -> Resources {
        self.current_resources.clone() + self.carryover_resources.clone()
    }

    /// Deduct `resources` from the available pool.
    ///
    /// Carryover resources are consumed first; any remainder is taken from
    /// current resources. Returns an error (and reverts all changes) if the
    /// total balance is insufficient.
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

    /// Add `resources` to the carryover pool (e.g. earnings from a compiled program run).
    pub fn give_carryover_resources(&mut self, resources: Resources) {
        self.carryover_resources += resources;
    }

    pub fn prestige(&mut self) {
        let convert = |x| std::cmp::min_by(x - 1e6, 0.0, f64::total_cmp).log10();
        let current_stars = self.current_resources.stars + self.carryover_resources.stars;

        self.carryover_resources = Resources::new(
            0.0,
            convert(self.current_resources.bronze.0),
            convert(self.current_resources.silver.0),
            convert(self.current_resources.gold.0),
            convert(self.current_resources.diamond.0),
        );

        self.current_resources = Resources::zero();
        self.current_resources.stars = current_stars;
    }
}

use crate::game_scenes::logic::audio::with_audio_backend_mut;
use crate::game_scenes::logic::auto_run::with_auto_run_mut;
use crate::game_state::upgrades::{Upgrade, Upgrades};
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
    pub compiled_program: Option<Result<CompiledProgram, (String, Vec<Vec<u64>>)>>,
    // Resources
    pub current_resources: Resources,
    pub carryover_resources: Resources,
    // Upgrades
    pub upgrades: Upgrades,
}

impl Default for GameState {
    fn default() -> Self {
        let start_code = r#""#;
        // let start_resources = Resources::new(1e9, 1e9, 1e9, 1e9, 1e9);
        let start_resources = Resources::zero();
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

    pub fn prestige_currency(&self) -> (Resources, Resources) {
        // Convert currency
        let convert = |x, min| {
            if x < min {
                0.0
            } else {
                let y = f64::log2(x).floor();
                if y.is_finite() { y } else { 0.0 }
            }
        };
        let current_stars = self.current_resources.stars + self.carryover_resources.stars;

        let carryover_resources = Resources::new(
            0.0,
            convert(self.current_resources.bronze.0, 1.),
            convert(self.current_resources.silver.0, 10.),
            convert(self.current_resources.gold.0, 100.),
            convert(self.current_resources.diamond.0, 1000.),
        );

        let mut current_resources = Resources::zero();
        current_resources.stars = current_stars;

        // Add currency from upgrades
        current_resources += self.upgrades.resources_after_reboot.value();

        (current_resources, carryover_resources)
    }

    fn prestige_upgrades(&self) -> Upgrades {
        // Reset upgrades
        let old_upgrades = self.upgrades.clone();
        let mut prestiged_upgrades = Upgrades::default();

        // Restore upgrades based on "keep upgrade"
        macro_rules! restore_upgrade {
            ($old:expr, $new:expr) => {
                for track_i in 0..$new.count_tracks() {
                    while $old.track_get_level(track_i) > $new.track_get_level(track_i) {
                        $new.track_level_up(track_i);
                    }
                }
            };
        }
        let keep_upgrades_until_group = old_upgrades.keep_prestige_upgrades.value();
        for upgrade_i in 0..Upgrades::UPGRADES_LEN {
            let upgrade = prestiged_upgrades.upgrade_at_mut(upgrade_i);
            let keep_upgrade = upgrade.group() <= keep_upgrades_until_group as usize;
            if keep_upgrade {
                let old_upgrade = old_upgrades.upgrades()[upgrade_i];
                restore_upgrade!(old_upgrade, upgrade);
            }
        }

        // Restore group unlocks
        restore_upgrade!(old_upgrades.unlock_level1, prestiged_upgrades.unlock_level1);
        restore_upgrade!(old_upgrades.unlock_level2, prestiged_upgrades.unlock_level2);
        restore_upgrade!(old_upgrades.unlock_level3, prestiged_upgrades.unlock_level3);
        restore_upgrade!(old_upgrades.unlock_level4, prestiged_upgrades.unlock_level4);
        restore_upgrade!(old_upgrades.unlock_level5, prestiged_upgrades.unlock_level5);
        restore_upgrade!(old_upgrades.unlock_level6, prestiged_upgrades.unlock_level6);

        prestiged_upgrades
    }

    pub fn prestige(&mut self) {
        (self.current_resources, self.carryover_resources) = self.prestige_currency();
        self.upgrades = self.prestige_upgrades();
        self.program_code = String::new();
        self.compiled_program = None;
        self.on_upgrades_commit();
    }

    pub fn on_upgrades_commit(&mut self) {
        // If music was unlocked, start the music.
        let unlock_music = self.upgrades.unlock_music.value();
        with_audio_backend_mut(|audio| {
            if let Some(audio) = audio {
                if unlock_music {
                    let _ = audio
                        .start_bgm_loop()
                        .map_err(|e| log::warn!("Error starting bgm: {}", e));
                } else {
                    audio.stop_bgm();
                }
            }
        });

        // If the instruction limit changed and the program was compiled already, re-compile it to recount the instructions
        if matches!(self.compiled_program, Some(Err(_))) {
            // TODO: for the moment, we just force the user to recompile manually
            self.compiled_program = None;
        }

        // Update the auto runner
        let auto_run_duration = self.upgrades.auto_run.value();
        with_auto_run_mut(|auto_run| {
            if let Some(auto_run_duration) = auto_run_duration {
                auto_run.set_period(auto_run_duration);
            } else {
                auto_run.stop();
            }
        });

        // If the win condition was bought, win
        if self.upgrades.win_condition.value() {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_resources_close(actual: &Resources, expected: &Resources) {
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        assert!(
            close(actual.bronze.0, expected.bronze.0)
                && close(actual.silver.0, expected.silver.0)
                && close(actual.gold.0, expected.gold.0)
                && close(actual.diamond.0, expected.diamond.0)
                && close(actual.stars.0, expected.stars.0),
            "expected {:?}, got {:?}",
            expected,
            actual
        );
    }

    #[test]
    fn currency_shifts_each_denomination_up_one_tier_1() {
        let mut state = GameState::default();
        state.current_resources = Resources::from_bronze(50);
        state.carryover_resources = Resources::zero();

        state.prestige_currency();

        assert_resources_close(&state.current_resources, &Resources::zero());
        assert_resources_close(&state.carryover_resources, &Resources::from_silver(2));
    }

    // Each denomination's log10 is written one tier up (bronze -> new silver,
    // silver -> new gold, gold -> new diamond, diamond -> new stars), and the
    // new bronze tier is always emptied. Carryover resources held before the
    // prestige are discarded (see `carryover_stars_and_discarded_carryover`
    // below for the one field, stars, that survives).
    #[test]
    fn currency_shifts_each_denomination_up_one_tier() {
        let mut state = GameState::default();
        state.current_resources = Resources::new(100.0, 1_000.0, 10.0, 1.0, 0.0);
        state.carryover_resources = Resources::zero();

        state.prestige_currency();

        assert_resources_close(&state.current_resources, &Resources::zero());
        assert_resources_close(
            &state.carryover_resources,
            &Resources::new(0.0, 2.0, 3.0, 1.0, 0.0),
        );
    }

    // Stars are the one resource that isn't reset: prior current + carryover
    // stars carry into the new current_resources untouched, and any log10 of
    // diamond adds *more* stars into carryover (diamond is the top tier, so
    // it feeds the prestige currency). Non-star carryover from before the
    // prestige (e.g. old carryover bronze) is discarded, not converted.
    #[test]
    fn carryover_stars_and_discarded_carryover() {
        let mut state = GameState::default();
        state.current_resources = Resources::new(1.0, 1.0, 1.0, 1_000.0, 5.0);
        state.carryover_resources = Resources::new(999.0, 0.0, 0.0, 0.0, 3.0);

        state.prestige_currency();

        assert_eq!(state.current_resources.stars.0, 8.0); // 5 + 3
        assert_eq!(state.carryover_resources.bronze.0, 0.0); // old 999 discarded
        assert_eq!(state.carryover_resources.stars.0, 3.0); // log10(1000) diamond -> stars
        assert_eq!(state.total_resources().stars.0, 11.0); // 8 (current) + 3 (carryover)
    }

    // resources_after_reboot is added to current_resources *after* it has
    // been zeroed, so it's the only source of non-star current_resources
    // right after a prestige.
    #[test]
    fn resources_after_reboot_upgrade_is_added_to_current() {
        let mut state = GameState::default();
        state.current_resources = Resources::zero();
        state.carryover_resources = Resources::zero();
        state.upgrades.resources_after_reboot.track_level_up(0);
        state.upgrades.resources_after_reboot.track_level_up(0);
        assert_eq!(
            state.upgrades.resources_after_reboot.value(),
            Resources::new(10_000.0, 100.0, 0.0, 0.0, 0.0)
        );

        state.prestige_currency();

        assert_resources_close(
            &state.current_resources,
            &Resources::new(10_000.0, 100.0, 0.0, 0.0, 0.0),
        );
    }

    // With the default keep_prestige_upgrades level (group 0, "keep L0"),
    // only group-0 upgrades survive a prestige, except the unlock_levelN
    // upgrades, which are always restored regardless of the setting.
    #[test]
    fn default_keep_level_only_preserves_group_zero_and_level_unlocks() {
        let mut state = GameState::default();
        state.upgrades.unlock_hud.track_level_up(0); // group 0
        state.upgrades.compile_time.track_level_up(0); // group 1
        state.upgrades.compile_time.track_level_up(0);
        state.upgrades.compile_time.track_level_up(0);
        state.upgrades.unlock_level2.track_level_up(0); // group 1, but always kept

        state.prestige_upgrades();

        assert!(state.upgrades.unlock_hud.value(), "group 0 upgrade kept");
        assert_eq!(
            state.upgrades.compile_time.value(),
            10.0,
            "group 1 upgrade reset to its default (level 0) value"
        );
        assert!(
            state.upgrades.unlock_level2.value(),
            "unlock_levelN is always preserved"
        );
    }

    // Raising keep_prestige_upgrades extends which groups (including itself,
    // group 2) survive a prestige; groups above the threshold still reset.
    #[test]
    fn keep_prestige_upgrades_raises_the_preserved_group_threshold() {
        let mut state = GameState::default();
        state.upgrades.keep_prestige_upgrades.track_level_up(0);
        state.upgrades.keep_prestige_upgrades.track_level_up(0); // level 2, "keep L2"
        state.upgrades.bronze_per_instruction.track_level_up(0); // group 2
        state.upgrades.bronze_per_instruction.track_level_up(0);
        state.upgrades.auto_compile.track_level_up(0); // group 3

        state.prestige_upgrades();

        assert_eq!(
            state.upgrades.keep_prestige_upgrades.value(),
            2,
            "keep_prestige_upgrades (group 2) preserves itself at group threshold 2"
        );
        assert_eq!(
            state.upgrades.bronze_per_instruction.track_get_level(0),
            2,
            "group 2 upgrade kept"
        );
        assert!(
            !state.upgrades.auto_compile.value(),
            "group 3 upgrade reset, above the threshold"
        );
    }

    // keep_prestige_upgrades is itself a group-2 upgrade, so setting it below
    // group 2 causes it to reset itself back to "keep L0" on the next
    // prestige.
    #[test]
    fn keep_prestige_upgrades_resets_itself_when_threshold_excludes_its_own_group() {
        let mut state = GameState::default();
        state.upgrades.keep_prestige_upgrades.track_level_up(0); // level 1, "keep L1"

        state.prestige_upgrades();

        assert_eq!(state.upgrades.keep_prestige_upgrades.value(), 0);
    }

    // Multi-track upgrades restore each track's level independently, not
    // just the combined total.
    #[test]
    fn kept_multi_track_upgrade_preserves_each_track_independently() {
        let mut state = GameState::default();
        state.upgrades.keep_prestige_upgrades.track_level_up(0); // level 1, "keep L1"
        state.upgrades.instruction_execution_speed.track_level_up(0); // group 1
        state.upgrades.instruction_execution_speed.track_level_up(0);
        state.upgrades.instruction_execution_speed.track_level_up(2);

        state.prestige_upgrades();

        assert_eq!(
            state
                .upgrades
                .instruction_execution_speed
                .track_get_level(0),
            2
        );
        assert_eq!(
            state
                .upgrades
                .instruction_execution_speed
                .track_get_level(2),
            1
        );
        assert_eq!(
            state
                .upgrades
                .instruction_execution_speed
                .track_get_level(1),
            0
        );
    }

    // The public prestige() entry point wires prestige_currency and
    // prestige_upgrades together.
    #[test]
    fn prestige_applies_both_currency_and_upgrade_changes() {
        let mut state = GameState::default();
        state.current_resources = Resources::new(100.0, 1.0, 1.0, 1.0, 0.0);
        state.carryover_resources = Resources::zero();
        state.upgrades.compile_time.track_level_up(0); // group 1, no keep upgrade purchased

        state.prestige();

        assert_resources_close(
            &state.current_resources,
            &Resources::zero(), // resources_after_reboot defaults to zero
        );
        assert_resources_close(
            &state.carryover_resources,
            &Resources::new(0.0, 2.0, 0.0, 0.0, 0.0), // log10(100) shifted into silver
        );
        assert_eq!(
            state.upgrades.compile_time.value(),
            10.0,
            "upgrades reset since keep_prestige_upgrades stayed at its default"
        );
    }
}

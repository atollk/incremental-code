use crate::game_state::{Resources, with_game_state};
use language::CompilingMetadata;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CompiledProgram {
    /// Number of instructions that were executed, separated by each call to `sleep`.
    pub instruction_counts: Vec<u64>,
    /// Calls to `sleep`, with their respective duration.
    pub sleep_calls: Vec<f64>,
    /// Calls to `print`, with their respective String lengths.
    pub print_calls: Vec<u64>,
    /// Calls to `brk`.
    pub brk_calls: u64,
}

const INSTRUCTION_BASIC_DURATION: Duration = Duration::from_millis(1000);

impl CompiledProgram {
    pub fn new() -> CompiledProgram {
        CompiledProgram {
            instruction_counts: vec![0],
            print_calls: vec![],
            sleep_calls: vec![],
            brk_calls: 0,
        }
    }

    pub fn instr_to_execution_time(instruction_counts: &[u64]) -> Duration {
        let constant_speed_up =
            with_game_state(|game_state| game_state.upgrades.instruction_execution_speed.value());
        instruction_counts
            .iter()
            .map(|&count| INSTRUCTION_BASIC_DURATION * count as u32)
            .map(|duration| duration / constant_speed_up)
            .sum()
    }

    pub fn execution_time(&self) -> Duration {
        Self::instr_to_execution_time(&self.instruction_counts)
    }

    pub fn resource_gain(&self) -> Resources {
        // TODO: silver=sleep, gold=print, diamond=brk
        struct ResourceUpgrades {
            bronze_per_instruction: (u8, f32),
            silver_per_sleep_second: u8,
            gold_per_print_character: u8,
            diamond_per_break: f32,
        }
        let upgrades = with_game_state(|game_state| ResourceUpgrades {
            bronze_per_instruction: game_state.upgrades.bronze_per_instruction.value(),
            silver_per_sleep_second: game_state.upgrades.silver_per_sleep_second.value(),
            gold_per_print_character: game_state.upgrades.gold_per_print_character.value(),
            diamond_per_break: game_state.upgrades.diamond_per_break.value(),
        });
        let (bronze_const, bronze_exp) = upgrades.bronze_per_instruction;
        let bronze = self
            .instruction_counts
            .iter()
            .map(|count| hurwitz(*count as f64, bronze_exp as f64) * bronze_const as f64)
            .sum();
        let silver = self
            .sleep_calls
            .iter()
            .map(|secs| secs * upgrades.silver_per_sleep_second as f64)
            .sum();
        let gold = self
            .print_calls
            .iter()
            .map(|len| {
                let mut x = *len as f64;
                for _ in 0..upgrades.gold_per_print_character {
                    x = x.log2().min(1.);
                }
                x
            })
            .sum();
        let diamond = todo!();
        Resources::new(bronze, silver, gold, diamond, 0.)
    }
}

// Approximates \sum_{i=1}^n i^k
fn hurwitz(n: f64, k: f64) -> f64 {
    let z = spfunc::zeta::zeta(-k);
    z + n.powf(k + 1.) / (k + 1.) + n.powf(k) / 2.
}

impl CompilingMetadata for CompiledProgram {
    fn log_zero_instruction(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn log_atomic_instruction(&mut self) -> anyhow::Result<()> {
        *self.instruction_counts.last_mut().unwrap() += 1;
        Ok(())
    }
}

use crate::game_state::{Resources, with_game_state};
use language::CompilingMetadata;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CompiledProgram {
    /// Number of instructions that were executed, separated by each call to `sleep`, respectively seperated by each call to `brk`.
    pub instruction_counts: Vec<Vec<u64>>,
    /// Calls to `sleep`, with their respective duration.
    pub sleep_calls: Vec<f64>,
    /// Last call to `print` with its string length.
    pub print_len: u64,
    /// Calls to `brk`.
    pub brk_calls: u64,
}

const INSTRUCTION_BASIC_DURATION: Duration = Duration::from_millis(1000);

impl CompiledProgram {
    pub fn new() -> CompiledProgram {
        CompiledProgram {
            instruction_counts: vec![vec![0]],
            sleep_calls: vec![],
            print_len: 0,
            brk_calls: 0,
        }
    }

    pub fn instr_to_execution_time(instruction_counts: &Vec<Vec<u64>>) -> Duration {
        struct Upgrades {
            instruction_execution_speed: (f32, f32),
            sleep_speed_reset: f32,
            min_instruction_duration: f32,
            break_slowdown: f32,
        }
        let upgrades = with_game_state(|game_state| Upgrades {
            instruction_execution_speed: game_state.upgrades.instruction_execution_speed.value(),
            sleep_speed_reset: game_state.upgrades.sleep_speed_reset.value(),
            min_instruction_duration: game_state.upgrades.min_instruction_duration.value(),
            break_slowdown: game_state.upgrades.break_slowdown.value(),
        });
        let (instruction_speed_const, instruction_speed_exp) = upgrades.instruction_execution_speed;
        let mut speed = 1.0;
        for brk_split in instruction_counts {
            let mut instruction_sum = 0;
            for sleep_split in brk_split {
                // TODO: use binary search to find the point between n=instruction_sum and n=(instruction_sum+sleep_split) at which the min_instruction_duration is reached
                let min_duration_reached_at = todo!();
                instruction_sum += sleep_split;
            }
        }
        // TODO
        todo!()
    }

    pub fn execution_time(&self) -> Duration {
        Self::instr_to_execution_time(&self.instruction_counts)
    }

    pub fn resource_gain(&self) -> Resources {
        struct ResourceUpgrades {
            bronze_per_instruction: (u8, f32),
            silver_per_sleep_second: u8,
            gold_per_print_character: u8,
            diamond_per_brk: u16,
        }
        let upgrades = with_game_state(|game_state| ResourceUpgrades {
            bronze_per_instruction: game_state.upgrades.bronze_per_instruction.value(),
            silver_per_sleep_second: game_state.upgrades.silver_per_sleep_second.value(),
            gold_per_print_character: game_state.upgrades.gold_per_print_character.value(),
            diamond_per_brk: game_state.upgrades.diamond_per_brk.value(),
        });
        let (bronze_const, bronze_exp) = upgrades.bronze_per_instruction;
        let bronze: f64 = {
            let counts = self
                .instruction_counts
                .iter()
                .map(|inner| inner.iter().map(|x| *x as f64).sum::<f64>())
                .sum();
            hurwitz(counts, bronze_exp as f64) * bronze_const as f64
        };
        let silver: f64 = self
            .sleep_calls
            .iter()
            .map(|secs| secs * upgrades.silver_per_sleep_second as f64)
            .sum();
        let gold: f64 = (0..upgrades.gold_per_print_character)
            .fold(self.print_len as f64, |acc, _| acc.log2().min(1.));
        let diamond = bronze.min(silver.min(gold)).log2() * upgrades.diamond_per_brk as f64;
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
        *self
            .instruction_counts
            .last_mut()
            .unwrap()
            .last_mut()
            .unwrap() += 1;
        Ok(())
    }
}

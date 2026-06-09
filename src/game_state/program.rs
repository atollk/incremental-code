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
            instruction_execution_speed: (f64, f64),
            sleep_speed_reset: f64,
            min_instruction_duration: f64,
            brk_slowdown: f64,
        }
        let upgrades = with_game_state(|game_state| Upgrades {
            instruction_execution_speed: game_state.upgrades.instruction_execution_speed.value(),
            sleep_speed_reset: game_state.upgrades.sleep_speed_reset.value(),
            min_instruction_duration: game_state.upgrades.min_instruction_duration.value(),
            brk_slowdown: game_state.upgrades.brk_slowdown.value(),
        });
        /*
        Math time:
        speed(s, n, k) : The instruction duration after n instructions, beginning from s, with k brks active. (not considering the min_instruction_duration)
        x,y: instruction_execution_speed
        b: brk_slowdown
        speed(s, n, k) = s * x * n^y / b^k

        Sum:
        duration(s, n, k) = sum_{i=1}^n speed(s, i, k) = s * x / b^k * sum_{i=1}^n i^y

        Inverse:
        n(speed, s, k) = (b^k * speed / s / x)^(1/y)
         */
        // Instruction Speed after n instructions and k brks: x*n^y / k^b (x,y: instruction_execution speed, b: brk_slowdown)
        let (instruction_speed_const, instruction_speed_exp) = upgrades.instruction_execution_speed;
        let mut speed = 1.0;
        let mut duration = 0.0;
        for (n_brks, brk_split) in instruction_counts.iter().enumerate() {
            let b_pow_k = upgrades.brk_slowdown.powi(n_brks as i32);
            for sleep_split in brk_split {
                let sleep_split = *sleep_split as f64;

                // Find the number of instructions at which min_instruction_duration is reached, if any.
                let min_duration_reached_at =
                    (b_pow_k * upgrades.min_instruction_duration / speed / instruction_speed_const)
                        .powf(1. / instruction_speed_exp);
                let min_duration_reached = min_duration_reached_at > sleep_split;

                // Compute the time taken
                let sleep_cycle_duration = if min_duration_reached {
                    let before_min = hurwitz(min_duration_reached_at, instruction_speed_exp)
                        * (speed)
                        * (instruction_speed_const)
                        / b_pow_k;
                    let after_min =
                        (sleep_split - min_duration_reached_at) * upgrades.min_instruction_duration;
                    before_min + after_min
                } else {
                    hurwitz(sleep_split, instruction_speed_exp)
                        * (speed)
                        * (instruction_speed_const)
                        / b_pow_k
                };
                duration += sleep_cycle_duration;

                // Compute new speed
                let pre_sleep_speed =
                    (speed * instruction_speed_const * sleep_split.powf(instruction_speed_exp)
                        / b_pow_k)
                        .max(upgrades.min_instruction_duration);
                speed = pre_sleep_speed.powf(upgrades.sleep_speed_reset);
            }
        }

        Duration::from_millis(1000).mul_f64(duration)
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

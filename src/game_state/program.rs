use crate::game_state::{Resources, with_game_state};
use anyhow::bail;
use itertools::Itertools;
use language::CompilingMetadata;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CompiledProgram {
    /// Number of instructions that were executed, separated by each call to `sleep`, respectively seperated by each call to `brk`.
    pub instruction_counts: Vec<Vec<u64>>,
    /// Calls to `sleep`, with their respective duration.
    pub sleep_calls: Vec<f64>,
    /// Max call to `print` with its string length.
    pub print_len: Option<u64>,
}

impl CompiledProgram {
    pub fn new() -> CompiledProgram {
        CompiledProgram {
            instruction_counts: vec![vec![0]],
            sleep_calls: vec![],
            print_len: None,
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
        let instruction_counts_between_brk = self
            .instruction_counts
            .iter()
            .map(|brk_set| brk_set.iter().map(|i| *i as f64).sum())
            .collect_vec();
        let total_instruction_count = instruction_counts_between_brk.iter().sum();
        // Bronze is awarded based on the total number of instructions run.
        let bronze: f64 = hurwitz(
            total_instruction_count,
            upgrades.bronze_per_instruction.1 as f64,
        ) * upgrades.bronze_per_instruction.0 as f64;
        // Silver is awarded based on the total sleep duration.
        let silver: f64 = self
            .sleep_calls
            .iter()
            .map(|secs| secs * upgrades.silver_per_sleep_second as f64)
            .sum();
        // Gold is awarded based on the last print statement and its argument length.
        let gold: f64 = (0..upgrades.gold_per_print_character)
            .fold(self.print_len.unwrap_or(0) as f64, |acc, _| {
                acc.log2().min(1.)
            });
        // Brk is awarded based on the other three resources, scaled by the point where the brk was called relative to all instructions.
        let diamond = {
            let brk_relatives = instruction_counts_between_brk
                .iter()
                .scan(0., |acc, x| Some(*acc + x))
                .map(|inst_cnt| 1. - (inst_cnt / total_instruction_count))
                .product::<f64>();
            bronze.min(silver.min(gold)).log2() * brk_relatives * upgrades.diamond_per_brk as f64
        };
        Resources::new(bronze, silver, gold, diamond, 0.)
    }
}

// Approximates \sum_{i=1}^n i^k
fn hurwitz(n: f64, k: f64) -> f64 {
    let z = spfunc::zeta::zeta(-k);
    z + n.powf(k + 1.) / (k + 1.) + n.powf(k) / 2.
}

impl CompilingMetadata for CompiledProgram {
    type Diff = CompiledProgram;

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

    fn diff(&self, other: &Self) -> anyhow::Result<Self::Diff> {
        let instruction_counts = vec![]; // TODO
        let sleep_calls = {
            for (l, r) in self.sleep_calls.iter().zip(other.sleep_calls.iter()) {
                if *l != *r {
                    bail!("sleep_calls mismatch");
                }
            }
            if self.sleep_calls.len() <= other.sleep_calls.len() {
                other.sleep_calls[self.sleep_calls.len()..].to_vec()
            } else {
                self.sleep_calls[other.sleep_calls.len()..].to_vec()
            }
        };
        let print_len = {
            if let Some(l) = self.print_len
                && other.print_len.map(|r| r < l).unwrap_or(true)
            {
                bail!("print_len mismatch");
            }
            other.print_len
        };
        Ok(CompiledProgram {
            instruction_counts,
            sleep_calls,
            print_len,
        })
    }

    fn add_assign(&mut self, diff: &Self::Diff) -> anyhow::Result<()> {
        {
            let (head, tail) = diff.instruction_counts.split_at(1);
            let head = head.iter().exactly_one().unwrap();
            let (h, t) = head.split_at(1);
            let h = *h.iter().exactly_one().unwrap();
            *self
                .instruction_counts
                .last_mut()
                .unwrap()
                .last_mut()
                .unwrap() += h;
            self.instruction_counts
                .last_mut()
                .unwrap()
                .extend_from_slice(t);
            self.instruction_counts.extend_from_slice(tail);
        };
        self.sleep_calls.extend(diff.sleep_calls.iter());
        self.print_len = self
            .print_len
            .map(|l| l.max(diff.print_len.unwrap_or(0)))
            .or(diff.print_len);
        Ok(())
    }
}

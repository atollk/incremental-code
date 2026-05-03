use crate::game_state::{Resources, with_game_state};
use language::CompilingMetadata;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CompiledProgram {
    pub instruction_counts: Vec<u64>,
}

// TODO: balancing
const INSTRUCTION_BASIC_DURATION: Duration = Duration::from_millis(10);

impl CompiledProgram {
    pub fn new() -> CompiledProgram {
        CompiledProgram {
            instruction_counts: vec![0],
        }
    }

    pub fn instr_to_execution_time(instruction_counts: &[u64]) -> Duration {
        let constant_speed_up = with_game_state(|game_state| {
            game_state
                .upgrades
                .speed_up_per_instruction_constant
                .current_value()
        });
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
        let bronze_linear = with_game_state(|game_state| {
            game_state
                .upgrades
                .bronze_per_instruction_linear
                .current_value()
        });
        let total_instructions: u64 = self.instruction_counts.iter().sum();
        Resources::from_bronze((total_instructions * bronze_linear as u64) as f64)
    }
}

impl CompilingMetadata for CompiledProgram {
    fn log_zero_instruction(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn log_atomic_instruction(&mut self) -> anyhow::Result<()> {
        // TODO: max instruction count check
        *self.instruction_counts.last_mut().unwrap() += 1;
        Ok(())
    }
}

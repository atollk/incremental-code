use crate::game_scenes::base::TickerMut;
use crate::game_state::{Resources, with_game_state, with_game_state_mut};
use crate::global_variable;
use std::time::Duration;

pub struct AutoRunner {
    period: Option<Duration>,
    run_duration: Option<Duration>,
    resource_gain: Resources,
    since_last_run: Duration,
}

impl Default for AutoRunner {
    fn default() -> Self {
        Self {
            period: None,
            run_duration: None,
            resource_gain: Resources::zero(),
            since_last_run: Duration::from_millis(0),
        }
    }
}

impl AutoRunner {
    pub fn set_period(&mut self, period: Duration) {
        self.period = Some(period);
    }

    pub fn stop(&mut self) {
        self.period = None;
    }
}

impl TickerMut for AutoRunner {
    fn tick(&mut self, elapsed: Duration) {
        if let Some(period) = self.period {
            self.since_last_run += elapsed;
            if let Some(run_duration) = self.run_duration {
                if self.since_last_run >= period + run_duration {
                    // Run finished: grant the resources
                    with_game_state_mut(|game_state| {
                        game_state.current_resources += self.resource_gain.clone();
                    });
                    self.run_duration = None;
                    self.resource_gain = Resources::zero();
                    self.since_last_run = Duration::from_secs(0);
                }
            } else {
                // Start the "run"
                (self.run_duration, self.resource_gain) = with_game_state(|game_state| {
                    if let Some(Ok(program)) = &game_state.compiled_program {
                        (Some(program.execution_time()), program.resource_gain())
                    } else {
                        (None, Resources::zero())
                    }
                });
            }
        }
    }
}

global_variable!(auto_run, AutoRunner);

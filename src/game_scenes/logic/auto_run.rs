use crate::game_scenes::base::TickerMut;
use crate::game_state::{Resources, with_game_state_mut};
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

    /// Progress (0.0-1.0) toward the next autorun reward, or `None` if autorun is inactive.
    pub fn get_progress(&self) -> Option<f64> {
        let period = self.period?;
        let deadline = period + self.run_duration.unwrap_or_default();
        if deadline.is_zero() {
            return Some(0.0);
        }
        Some((self.since_last_run.as_secs_f64() / deadline.as_secs_f64()).clamp(0.0, 1.0))
    }

    pub(crate) fn reset(&mut self) {
        let program_stats =
            with_game_state_mut(|game_state| game_state.get_or_compute_program_stats());
        (self.run_duration, self.resource_gain) = if let Some(program_stats) = program_stats {
            (Some(program_stats.1), program_stats.0)
        } else {
            (None, Resources::zero())
        }
    }

    fn on_timer_done(&mut self) {
        // Run finished: grant the resources
        with_game_state_mut(|game_state| {
            game_state.current_resources += self.resource_gain.clone();
            if game_state.upgrades.additive_reboot.value().1 {
                game_state.current_resources += game_state.prestige_currency().1;
            }
        });
        self.run_duration = None;
        self.resource_gain = Resources::zero();
        self.since_last_run = Duration::from_secs(0);
    }
}

impl TickerMut for AutoRunner {
    fn tick(&mut self, elapsed: Duration) {
        if let Some(period) = self.period {
            self.since_last_run += elapsed;
            if let Some(run_duration) = self.run_duration {
                if self.since_last_run >= period + run_duration {
                    self.on_timer_done();
                }
            } else {
                self.reset();
            }
        }
    }
}

global_variable!(auto_run, AutoRunner);

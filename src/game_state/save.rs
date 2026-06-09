use crate::backend::backend::StorageBackend;
use crate::backend::with_backend;
use crate::game_scenes::base::TickerMut;
use crate::game_state::{with_game_state, with_game_state_mut};
use crate::global_variable;
use web_time::Duration;

pub struct AutoSaver {
    since_last_save: Duration,
    save_period: Option<Duration>,
}

impl Default for AutoSaver {
    fn default() -> AutoSaver {
        Self {
            since_last_save: Duration::from_millis(0),
            save_period: None,
        }
    }
}

impl AutoSaver {
    fn save(&mut self) {
        self.since_last_save = Duration::from_millis(0);
        if let Err(e) = save_game_state() {
            log::error!("Auto-save failed: {e}");
        }
    }

    pub fn since_last_save(&self) -> Duration {
        self.since_last_save
    }

    pub fn start(&mut self, period: Duration) {
        self.save_period = Some(period);
        self.since_last_save = Duration::from_millis(0);
    }

    pub fn stop(&mut self) {
        self.save_period = None;
    }
}

global_variable!(auto_saver, AutoSaver);

impl TickerMut for AutoSaver {
    fn tick(&mut self, elapsed: Duration) {
        if let Some(save_period) = &self.save_period {
            self.since_last_save += elapsed;
            if self.since_last_save >= *save_period {
                self.save();
            }
        }
    }
}

const KEY: &'static str = "game_state";

pub fn save_game_state() -> anyhow::Result<()> {
    let storage_backend: impl StorageBackend = with_backend(|backend| backend.storage_backend());
    with_game_state(|game_state| storage_backend.save(KEY, game_state))?;
    Ok(())
}

pub fn load_game_state() -> anyhow::Result<bool> {
    let storage_backend: impl StorageBackend = with_backend(|backend| backend.storage_backend());
    let loaded_state = storage_backend.load(KEY)?;
    if let Some(state) = loaded_state {
        with_game_state_mut(|game_state| *game_state = state);
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn erase_game_state() -> anyhow::Result<()> {
    let storage_backend: impl StorageBackend = with_backend(|backend| backend.storage_backend());
    storage_backend.delete(KEY)
}

use crate::backend::backend::StorageBackend;
use crate::backend::with_backend;
use crate::game_state::{with_game_state, with_game_state_mut};
use std::sync::{LazyLock, Mutex};
use web_time::{Duration, Instant};

pub struct AutoSaver {
    last_save_time: Instant,
    save_period: Option<Duration>,
}

impl AutoSaver {
    fn new() -> Self {
        Self {
            last_save_time: Instant::now(),
            save_period: None,
        }
    }

    fn save(&mut self) {
        self.last_save_time = Instant::now();
        if let Err(e) = save_game_state() {
            log::error!("Auto-save failed: {e}");
        }
    }

    pub fn tick(&mut self) {
        if let Some(save_period) = &self.save_period {
            if (Instant::now() - self.last_save_time).ge(save_period) {
                self.save();
            }
        }
    }

    pub fn start(&mut self, period: Duration) {
        self.save_period = Some(period);
        self.save();
    }

    pub fn stop(&mut self) {
        self.save_period = None;
    }
}

pub static AUTO_SAVER: LazyLock<Mutex<AutoSaver>> = LazyLock::new(|| Mutex::new(AutoSaver::new()));

const KEY: &'static str = "save";

pub fn save_game_state() -> anyhow::Result<()> {
    let storage_backend: impl StorageBackend = with_backend(|backend| backend.storage_backend());
    with_game_state(|game_state| storage_backend.save(KEY, game_state))?;
    Ok(())
}

pub fn load_game_state() -> anyhow::Result<()> {
    let storage_backend: impl StorageBackend = with_backend(|backend| backend.storage_backend());
    let loaded_state = storage_backend.load(KEY)?;
    if let Some(state) = loaded_state {
        with_game_state_mut(|game_state| *game_state = state);
    }
    Ok(())
}

pub fn erase_game_state() -> anyhow::Result<()> {
    let storage_backend: impl StorageBackend = with_backend(|backend| backend.storage_backend());
    storage_backend.delete(KEY)
}

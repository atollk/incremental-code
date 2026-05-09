use crate::backend::backend::StorageBackend;
use crate::backend::with_backend;
use parking_lot::ReentrantMutex;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::sync::LazyLock;

pub fn with_settings<T>(f: impl FnOnce(&Settings) -> T) -> T {
    let lock = GLOBAL_SETTINGS.lock();
    f(lock.deref().borrow().deref())
}

pub fn with_settings_mut<T>(f: impl FnOnce(&mut Settings) -> T) -> T {
    let lock = GLOBAL_SETTINGS.lock();
    f(lock.deref().borrow_mut().deref_mut())
}

static GLOBAL_SETTINGS: LazyLock<ReentrantMutex<RefCell<Settings>>> =
    LazyLock::new(|| ReentrantMutex::new(RefCell::new(Settings::default())));

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub(crate) bgm_volume: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self { bgm_volume: 0.1 }
    }
}

const KEY: &'static str = "settings";

pub fn save_settings() -> anyhow::Result<()> {
    let storage_backend: impl StorageBackend = with_backend(|backend| backend.storage_backend());
    with_settings(|settings| storage_backend.save(KEY, settings))?;
    Ok(())
}

pub fn load_settings() -> anyhow::Result<bool> {
    let storage_backend: impl StorageBackend = with_backend(|backend| backend.storage_backend());
    let loaded_state = storage_backend.load(KEY)?;
    if let Some(state) = loaded_state {
        with_settings_mut(|settings| *settings = state);
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn erase_settings() -> anyhow::Result<()> {
    let storage_backend: impl StorageBackend = with_backend(|backend| backend.storage_backend());
    storage_backend.delete(KEY)
}

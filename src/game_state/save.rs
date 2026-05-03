use crate::backend::backend::StorageBackend;
use crate::backend::with_backend;
use crate::game_state::{with_game_state, with_game_state_mut};

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

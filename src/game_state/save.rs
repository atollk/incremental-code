use crate::backend;
use crate::backend::backend::StorageBackend;
use crate::backend::with_backend;
use crate::game_state::with_game_state;

pub fn save_game_state() -> anyhow::Result<()> {
    let storage_backend = with_backend(|backend| backend.storage_backend());
    with_game_state(|game_state| storage_backend.save("save", game_state))?;
    Ok(())
}

use crate::backend;

pub fn save_game_state() {
    let backend_lock = backend::BACKEND_INSTANCE.lock();
}

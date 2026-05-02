use crate::backend::events::Event;
use serde::{Deserialize, Serialize};

/// A backend implementation that can own and drive a [`TerminalApp`] event loop.
pub trait BackendSuite<B: ratatui::backend::Backend, S: StorageBackend + Default> {
    /// Start the event loop and run `app` until it requests an exit.
    fn run(&self, app: &mut dyn TerminalApp<B>) -> anyhow::Result<()>;

    /// Enable logging.
    fn init_logging(&self) -> anyhow::Result<()>;

    /// StorageBackend to work with persisting data.
    fn storage_backend(&self) -> S {
        S::default()
    }
}

/// A backend to store data between runs.
pub trait StorageBackend {
    /// Persist data.
    fn save<T: Serialize>(&self, key: &str, data: &T) -> anyhow::Result<()>;

    /// Load persisted data.
    fn load<T: for<'a> Deserialize<'a>>(&self, key: &str) -> anyhow::Result<Option<T>>;

    /// Clear persisted data.
    fn delete(&self, key: &str) -> anyhow::Result<()>;
}

/// An application that the backend drives frame by frame.
pub trait TerminalApp<B: ratatui::backend::Backend> {
    /// Called once after the backend is ready, before the first frame.
    fn init(&mut self, backend: B) -> anyhow::Result<()>;

    /// Called every frame with the events collected since the last frame.
    ///
    /// Returns `true` to request a clean exit.
    fn frame(&mut self, events: &[Event]) -> anyhow::Result<bool>;

    /// Returns a shared reference to the underlying backend.
    #[allow(dead_code)]
    fn backend(&self) -> &B;

    /// Returns a mutable reference to the underlying backend.
    #[allow(dead_code)]
    fn backend_mut(&mut self) -> &mut B;
}

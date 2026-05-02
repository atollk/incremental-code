use crate::backend::events::Event;

/// A backend implementation that can own and drive a [`TerminalApp`] event loop.
pub trait BackendSuite<B: ratatui::backend::Backend> {
    /// Start the event loop and run `app` until it requests an exit.
    fn run(&mut self, app: impl TerminalApp<B> + 'static) -> anyhow::Result<()>;
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

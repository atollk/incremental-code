use crate::backend::events::Event;

pub trait BackendSuite<B: ratatui::backend::Backend> {
    fn run(&mut self, app: impl TerminalApp<B> + 'static) -> anyhow::Result<()>;
}

pub trait TerminalApp<B: ratatui::backend::Backend> {
    fn init(&mut self, backend: B) -> anyhow::Result<()>;
    fn frame(&mut self, events: &[Event]) -> anyhow::Result<bool>;
    #[allow(dead_code)]
    fn backend(&self) -> &B;
    #[allow(dead_code)]
    fn backend_mut(&mut self) -> &mut B;
}
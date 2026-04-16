
pub trait BackendSuite<B: ratatui::backend::Backend> {
    fn run(&mut self, app: impl TerminalApp<B>) -> anyhow::Result<()>;
}

pub trait TerminalApp<B: ratatui::backend::Backend> {
    fn init(&mut self, backend: B) -> anyhow::Result<()>;
    fn frame(&mut self) -> anyhow::Result<()>;
    fn backend(&self) -> &B;
    fn backend_mut(&self) -> &mut B;
}
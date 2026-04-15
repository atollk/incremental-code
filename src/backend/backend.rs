pub trait IntegrationBackend<B: ratatui::backend::Backend> {
    fn run(&mut self, runner: impl FnMut(B)) -> anyhow::Result<()>;
}

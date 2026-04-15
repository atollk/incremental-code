use crate::backend::backend::IntegrationBackend;
use ratatui::backend::CrosstermBackend;
use std::io::Stdout;
use std::sync::LazyLock;

#[derive(Default)]
pub struct CrosstermIntegrationBackend {}

impl IntegrationBackend<CrosstermBackend<Stdout>> for CrosstermIntegrationBackend {
    fn run(&mut self, mut runner: impl FnMut(CrosstermBackend<Stdout>)) -> anyhow::Result<()> {
        let backend = CrosstermBackend::new(std::io::stdout());
        runner(backend);
        Ok(())
    }
}

pub static BACKEND_INSTANCE: LazyLock<CrosstermIntegrationBackend> =
    LazyLock::new(|| CrosstermIntegrationBackend {});

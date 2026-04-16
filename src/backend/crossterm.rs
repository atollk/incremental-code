use crate::backend::backend::BackendSuite;
use ratatui::backend::CrosstermBackend;
use std::io::Stdout;
use std::sync::LazyLock;

#[derive(Default)]
pub struct CrosstermBackendSuite {}

impl BackendSuite<CrosstermBackend<Stdout>> for CrosstermBackendSuite {
    fn run(&mut self, mut runner: impl FnMut(CrosstermBackend<Stdout>)) -> anyhow::Result<()> {
        let backend = CrosstermBackend::new(std::io::stdout());
        runner(backend);
        Ok(())
    }
}

pub static BACKEND_INSTANCE: LazyLock<CrosstermBackendSuite> =
    LazyLock::new(|| CrosstermBackendSuite {});

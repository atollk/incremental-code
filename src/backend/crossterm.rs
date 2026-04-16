use crate::backend::backend::{BackendSuite, TerminalApp};
use ratatui::backend::CrosstermBackend;
use std::io::Stdout;
use std::sync::{LazyLock, Mutex};
use glow::BACK;
use crate::backend::beamterm_native::BeamtermCoreBackendSuite;

pub type BackendType = CrosstermBackend<Stdout>;

pub static BACKEND_INSTANCE: LazyLock<Mutex<CrosstermBackendSuite>> =
    LazyLock::new(|| Mutex::new(CrosstermBackendSuite {}));

#[derive(Default)]
pub struct CrosstermBackendSuite {}

impl BackendSuite<BackendType> for CrosstermBackendSuite {
    fn run(&mut self, mut app: impl TerminalApp<BackendType>) -> anyhow::Result<()> {
        let backend = BackendType::new(std::io::stdout());
        app.init(backend)?;
        loop {
            app.frame()?;
        }
    }
}

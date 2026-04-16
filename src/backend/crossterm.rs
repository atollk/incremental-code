use crate::backend::backend::{BackendSuite, TerminalApp};
use ratatui::backend::CrosstermBackend;
use std::io::{stdout, Stdout};
use std::sync::{LazyLock, Mutex};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};

pub type BackendType = CrosstermBackend<Stdout>;

pub static BACKEND_INSTANCE: LazyLock<Mutex<CrosstermBackendSuite>> =
    LazyLock::new(|| Mutex::new(CrosstermBackendSuite {}));

#[derive(Default)]
pub struct CrosstermBackendSuite {}

impl BackendSuite<BackendType> for CrosstermBackendSuite {
    fn run(&mut self, mut app: impl TerminalApp<BackendType> + 'static) -> anyhow::Result<()> {
        let backend = BackendType::new(std::io::stdout());
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        app.init(backend)?;
        loop {
            app.frame()?;
        }
    }
}

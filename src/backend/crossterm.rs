use crate::backend::backend::{BackendSuite, TerminalApp};
use ratatui::backend::CrosstermBackend;
use std::io::{stdout, Stdout};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use crate::backend::events::{Event, IntoEvent};

pub type BackendType = CrosstermBackend<Stdout>;

pub static BACKEND_INSTANCE: LazyLock<Mutex<CrosstermBackendSuite>> =
    LazyLock::new(|| Mutex::new(CrosstermBackendSuite {}));

#[derive(Default)]
pub struct CrosstermBackendSuite {}

impl BackendSuite<BackendType> for CrosstermBackendSuite {
    fn run(&mut self, mut app: impl TerminalApp<BackendType> + 'static) -> anyhow::Result<()> {
        let backend = BackendType::new(stdout());
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        app.init(backend)?;

        let events = {
            let mut events = Vec::new();
            while crossterm::event::poll(Duration::from_millis(0))? {
                let event = crossterm::event::read()?;
                if let Some(event) = event.into_event() {
                    events.push(event);
                }
            }
            events
        };
        
        loop {
            app.frame(&events)?;
        }
    }
}

impl IntoEvent for crossterm::event::Event {
    fn into_event(self) -> Option<Event> {
        todo!()
    }
}
use crate::backend::backend::{BackendSuite, StorageBackend, TerminalApp};
use crate::backend::events::{Event, IntoEvent};
use crate::backend::store_native::StoreNative;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use log::LevelFilter;
use ratatui::backend::CrosstermBackend;
use std::io::{Stdout, stdout};
use std::sync::{LazyLock, Mutex, RwLock};
use std::time::Duration;

pub type BackendType = CrosstermBackend<Stdout>;
pub type StorageType = StoreNative;

pub static BACKEND_INSTANCE: LazyLock<RwLock<CrosstermBackendSuite>> =
    LazyLock::new(|| RwLock::new(CrosstermBackendSuite {}));

#[derive(Default)]
pub struct CrosstermBackendSuite {}

impl BackendSuite<BackendType, StorageType> for CrosstermBackendSuite {
    fn run(&self, app: &mut dyn TerminalApp<BackendType>) -> anyhow::Result<()> {
        let backend = BackendType::new(stdout());
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        app.init(backend)?;

        let mut events = Vec::new();
        let mut exit = false;
        while !exit {
            while crossterm::event::poll(Duration::from_millis(0))? {
                let event = crossterm::event::read()?;
                if let Some(event) = event.into_event() {
                    events.push(event);
                }
            }
            exit = app.frame(&events)?;
            events.clear();
        }
        Ok(())
    }

    fn init_logging(&self) -> anyhow::Result<()> {
        simple_logger::SimpleLogger::new()
            .with_level(LevelFilter::Debug)
            .init()
            .map_err(|e| anyhow::anyhow!(e))
    }
}

impl IntoEvent for crossterm::event::Event {
    fn into_event(self) -> Option<Event> {
        use crate::backend::input::{
            KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
            MouseEventKind,
        };
        use crossterm::event as ct;

        fn map_button(b: ct::MouseButton) -> MouseButton {
            match b {
                ct::MouseButton::Left => MouseButton::Left,
                ct::MouseButton::Right => MouseButton::Right,
                ct::MouseButton::Middle => MouseButton::Middle,
            }
        }

        match self {
            ct::Event::Key(k) => {
                let code = match k.code {
                    ct::KeyCode::Backspace => KeyCode::Backspace,
                    ct::KeyCode::Enter => KeyCode::Enter,
                    ct::KeyCode::Left => KeyCode::Left,
                    ct::KeyCode::Right => KeyCode::Right,
                    ct::KeyCode::Up => KeyCode::Up,
                    ct::KeyCode::Down => KeyCode::Down,
                    ct::KeyCode::Home => KeyCode::Home,
                    ct::KeyCode::End => KeyCode::End,
                    ct::KeyCode::PageUp => KeyCode::PageUp,
                    ct::KeyCode::PageDown => KeyCode::PageDown,
                    ct::KeyCode::Tab => KeyCode::Tab,
                    ct::KeyCode::BackTab => KeyCode::BackTab,
                    ct::KeyCode::Delete => KeyCode::Delete,
                    ct::KeyCode::Insert => KeyCode::Insert,
                    ct::KeyCode::F(n) => KeyCode::F(n),
                    ct::KeyCode::Char(c) => KeyCode::Char(c),
                    ct::KeyCode::Null => KeyCode::Null,
                    ct::KeyCode::Esc => KeyCode::Esc,
                    ct::KeyCode::CapsLock => KeyCode::CapsLock,
                    ct::KeyCode::ScrollLock => KeyCode::ScrollLock,
                    ct::KeyCode::NumLock => KeyCode::NumLock,
                    ct::KeyCode::PrintScreen => KeyCode::PrintScreen,
                    ct::KeyCode::Pause => KeyCode::Pause,
                    ct::KeyCode::Menu => KeyCode::Menu,
                    ct::KeyCode::KeypadBegin => KeyCode::KeypadBegin,
                    _ => return None,
                };
                let modifiers = KeyModifiers::from_bits_truncate(k.modifiers.bits());
                let kind = match k.kind {
                    ct::KeyEventKind::Press => KeyEventKind::Press,
                    ct::KeyEventKind::Repeat => KeyEventKind::Repeat,
                    ct::KeyEventKind::Release => KeyEventKind::Release,
                };
                let state = KeyEventState::from_bits_truncate(k.state.bits());
                Some(Event::KeyEvent(KeyEvent {
                    code,
                    modifiers,
                    kind,
                    state,
                }))
            }
            ct::Event::Mouse(m) => {
                let kind = match m.kind {
                    ct::MouseEventKind::Down(b) => MouseEventKind::Down(map_button(b)),
                    ct::MouseEventKind::Up(b) => MouseEventKind::Up(map_button(b)),
                    ct::MouseEventKind::Drag(b) => MouseEventKind::Drag(map_button(b)),
                    ct::MouseEventKind::Moved => MouseEventKind::Moved,
                    ct::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
                    ct::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
                    ct::MouseEventKind::ScrollLeft => MouseEventKind::ScrollLeft,
                    ct::MouseEventKind::ScrollRight => MouseEventKind::ScrollRight,
                };
                let modifiers = KeyModifiers::from_bits_truncate(m.modifiers.bits());
                Some(Event::MouseEvent(MouseEvent {
                    kind,
                    column: m.column,
                    row: m.row,
                    modifiers,
                }))
            }
            _ => None,
        }
    }
}

use crate::backend::backend::{BackendSuite, StorageBackend, TerminalApp};
use crate::backend::events::{Event, IntoEvent};
use crate::backend::store_native::StoreNative;
use crate::backend::store_web::StoreWeb;
use ratzilla::event::{
    KeyCode as RzKeyCode, MouseButton as RzMouseButton, MouseEventKind as RzMouseEventKind,
};
use ratzilla::web_sys::wasm_bindgen::JsCast;
use ratzilla::web_sys::wasm_bindgen::closure::Closure;
use ratzilla::{WebEventHandler, WebGl2Backend};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{LazyLock, Mutex};

pub type BackendType = WebGl2Backend;

pub static BACKEND_INSTANCE: LazyLock<Mutex<RatzillaBackendSuite>> =
    LazyLock::new(|| Mutex::new(RatzillaBackendSuite {}));

pub struct RatzillaBackendSuite {}

impl BackendSuite<BackendType> for RatzillaBackendSuite {
    fn run(
        &mut self,
        mut terminal_app: impl TerminalApp<BackendType> + 'static,
    ) -> anyhow::Result<()> {
        let mut backend = WebGl2Backend::new().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let events: Rc<RefCell<Vec<Event>>> = Rc::new(RefCell::new(Vec::new()));

        {
            let events = events.clone();
            backend
                .on_key_event(move |key_event| {
                    if let Some(event) = key_event.into_event() {
                        events.borrow_mut().push(event);
                    }
                })
                .ok();
        }

        {
            let events = events.clone();
            backend
                .on_mouse_event(move |mouse_event| {
                    if let Some(event) = mouse_event.into_event() {
                        events.borrow_mut().push(event);
                    }
                })
                .ok();
        }

        terminal_app.init(backend)?;

        let terminal_app = Rc::new(RefCell::new(terminal_app));

        let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let current_events: Vec<Event> = events.borrow_mut().drain(..).collect();
            let exit = terminal_app
                .borrow_mut()
                .frame(&current_events)
                .unwrap_or(true);
            if !exit {
                ratzilla::web_sys::window()
                    .unwrap()
                    .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                    .unwrap();
            }
        }) as Box<dyn FnMut()>));

        ratzilla::web_sys::window()
            .unwrap()
            .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();

        Ok(())
    }

    fn storage_backend(&self) -> impl StorageBackend {
        StoreWeb::default()
    }
}

impl IntoEvent for ratzilla::event::KeyEvent {
    fn into_event(self) -> Option<Event> {
        use crate::backend::input::{
            KeyCode, KeyEvent as IKeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
        };

        let code = match self.code {
            RzKeyCode::Char(c) => KeyCode::Char(c),
            RzKeyCode::F(n) => KeyCode::F(n),
            RzKeyCode::Backspace => KeyCode::Backspace,
            RzKeyCode::Enter => KeyCode::Enter,
            RzKeyCode::Left => KeyCode::Left,
            RzKeyCode::Right => KeyCode::Right,
            RzKeyCode::Up => KeyCode::Up,
            RzKeyCode::Down => KeyCode::Down,
            RzKeyCode::Tab => KeyCode::Tab,
            RzKeyCode::Delete => KeyCode::Delete,
            RzKeyCode::Home => KeyCode::Home,
            RzKeyCode::End => KeyCode::End,
            RzKeyCode::PageUp => KeyCode::PageUp,
            RzKeyCode::PageDown => KeyCode::PageDown,
            RzKeyCode::Esc => KeyCode::Esc,
            RzKeyCode::Unidentified => return None,
        };

        let mut modifiers = KeyModifiers::NONE;
        if self.ctrl {
            modifiers |= KeyModifiers::CONTROL;
        }
        if self.alt {
            modifiers |= KeyModifiers::ALT;
        }
        if self.shift {
            modifiers |= KeyModifiers::SHIFT;
        }

        Some(Event::KeyEvent(IKeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }))
    }
}

impl IntoEvent for ratzilla::event::MouseEvent {
    fn into_event(self) -> Option<Event> {
        use crate::backend::input::{
            KeyModifiers, MouseButton, MouseEvent as IMouseEvent, MouseEventKind,
        };

        fn map_button(b: RzMouseButton) -> Option<MouseButton> {
            match b {
                RzMouseButton::Left => Some(MouseButton::Left),
                RzMouseButton::Right => Some(MouseButton::Right),
                RzMouseButton::Middle => Some(MouseButton::Middle),
                RzMouseButton::Back | RzMouseButton::Forward | RzMouseButton::Unidentified => None,
            }
        }

        let kind = match self.kind {
            RzMouseEventKind::ButtonDown(b) => MouseEventKind::Down(map_button(b)?),
            RzMouseEventKind::ButtonUp(b) => MouseEventKind::Up(map_button(b)?),
            RzMouseEventKind::SingleClick(b) => MouseEventKind::Down(map_button(b)?),
            RzMouseEventKind::Moved => MouseEventKind::Moved,
            RzMouseEventKind::DoubleClick(_)
            | RzMouseEventKind::Entered
            | RzMouseEventKind::Exited
            | RzMouseEventKind::Unidentified => return None,
        };

        let mut modifiers = KeyModifiers::NONE;
        if self.ctrl {
            modifiers |= KeyModifiers::CONTROL;
        }
        if self.alt {
            modifiers |= KeyModifiers::ALT;
        }
        if self.shift {
            modifiers |= KeyModifiers::SHIFT;
        }

        Some(Event::MouseEvent(IMouseEvent {
            kind,
            column: self.col,
            row: self.row,
            modifiers,
        }))
    }
}

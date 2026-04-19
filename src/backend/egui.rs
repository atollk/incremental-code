#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::backend::backend::{BackendSuite, TerminalApp};
use crate::backend::events::{Event, IntoEvent};
use eframe::{CreationContext, Frame};
use egui_ratatui::RataguiBackend;
use soft_ratatui::embedded_graphics_unicodefonts::{
    mono_8x13_atlas, mono_8x13_bold_atlas, mono_8x13_italic_atlas,
};
use soft_ratatui::{EmbeddedGraphics, SoftBackend};
use std::sync::{LazyLock, Mutex};

pub type BackendType = RataguiBackend<EmbeddedGraphics>;

pub static BACKEND_INSTANCE: LazyLock<Mutex<EguiBackendSuite>> =
    LazyLock::new(|| Mutex::new(EguiBackendSuite {}));

pub struct EguiBackendSuite {}

impl EguiBackendSuite {
    pub(crate) fn make_backend(&self) -> BackendType {
        let font_regular = mono_8x13_atlas();
        let font_italic = mono_8x13_italic_atlas();
        let font_bold = mono_8x13_bold_atlas();
        let soft_backend = SoftBackend::<EmbeddedGraphics>::new(
            100,
            50,
            font_regular,
            Some(font_bold),
            Some(font_italic),
        );
        let backend = BackendType::new("soft_rat", soft_backend);
        backend
    }
}

pub struct EguiApplicationHandler<A: TerminalApp<BackendType>> {
    terminal_app: A,
}

impl<A: TerminalApp<BackendType>> EguiApplicationHandler<A> {
    pub fn new(_cc: &CreationContext<'_>, terminal_app: A) -> Self {
        EguiApplicationHandler { terminal_app }
    }
}

impl<A: TerminalApp<BackendType>> eframe::App for EguiApplicationHandler<A> {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        let events = ui.input(parse_input_events);
        let exit = self.terminal_app.frame(&events).unwrap();
        if exit {
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add(self.terminal_app.backend_mut());
        });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}
}

fn parse_input_events(input: &egui::InputState) -> Vec<Event> {
    input
        .events
        .iter()
        .cloned()
        .filter_map(IntoEvent::into_event)
        .collect()
}

impl IntoEvent for egui::Event {
    fn into_event(self) -> Option<Event> {
        use crate::backend::input::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

        match self {
            egui::Event::Key {
                key,
                pressed,
                repeat,
                modifiers,
                ..
            } => {
                let code = match key {
                    egui::Key::ArrowDown => KeyCode::Down,
                    egui::Key::ArrowLeft => KeyCode::Left,
                    egui::Key::ArrowRight => KeyCode::Right,
                    egui::Key::ArrowUp => KeyCode::Up,
                    egui::Key::Escape => KeyCode::Esc,
                    egui::Key::Tab => KeyCode::Tab,
                    egui::Key::Backspace => KeyCode::Backspace,
                    egui::Key::Enter => KeyCode::Enter,
                    egui::Key::Insert => KeyCode::Insert,
                    egui::Key::Delete => KeyCode::Delete,
                    egui::Key::Home => KeyCode::Home,
                    egui::Key::End => KeyCode::End,
                    egui::Key::PageUp => KeyCode::PageUp,
                    egui::Key::PageDown => KeyCode::PageDown,
                    egui::Key::Space => KeyCode::Char(' '),
                    egui::Key::Colon => KeyCode::Char(':'),
                    egui::Key::Comma => KeyCode::Char(','),
                    egui::Key::Backslash => KeyCode::Char('\\'),
                    egui::Key::Slash => KeyCode::Char('/'),
                    egui::Key::Pipe => KeyCode::Char('|'),
                    egui::Key::Questionmark => KeyCode::Char('?'),
                    egui::Key::Exclamationmark => KeyCode::Char('!'),
                    egui::Key::OpenBracket => KeyCode::Char('['),
                    egui::Key::CloseBracket => KeyCode::Char(']'),
                    egui::Key::OpenCurlyBracket => KeyCode::Char('{'),
                    egui::Key::CloseCurlyBracket => KeyCode::Char('}'),
                    egui::Key::Backtick => KeyCode::Char('`'),
                    egui::Key::Minus => KeyCode::Char('-'),
                    egui::Key::Period => KeyCode::Char('.'),
                    egui::Key::Plus => KeyCode::Char('+'),
                    egui::Key::Equals => KeyCode::Char('='),
                    egui::Key::Semicolon => KeyCode::Char(';'),
                    egui::Key::Quote => KeyCode::Char('\''),
                    egui::Key::Num0 => KeyCode::Char('0'),
                    egui::Key::Num1 => KeyCode::Char('1'),
                    egui::Key::Num2 => KeyCode::Char('2'),
                    egui::Key::Num3 => KeyCode::Char('3'),
                    egui::Key::Num4 => KeyCode::Char('4'),
                    egui::Key::Num5 => KeyCode::Char('5'),
                    egui::Key::Num6 => KeyCode::Char('6'),
                    egui::Key::Num7 => KeyCode::Char('7'),
                    egui::Key::Num8 => KeyCode::Char('8'),
                    egui::Key::Num9 => KeyCode::Char('9'),
                    egui::Key::A => KeyCode::Char('a'),
                    egui::Key::B => KeyCode::Char('b'),
                    egui::Key::C => KeyCode::Char('c'),
                    egui::Key::D => KeyCode::Char('d'),
                    egui::Key::E => KeyCode::Char('e'),
                    egui::Key::F => KeyCode::Char('f'),
                    egui::Key::G => KeyCode::Char('g'),
                    egui::Key::H => KeyCode::Char('h'),
                    egui::Key::I => KeyCode::Char('i'),
                    egui::Key::J => KeyCode::Char('j'),
                    egui::Key::K => KeyCode::Char('k'),
                    egui::Key::L => KeyCode::Char('l'),
                    egui::Key::M => KeyCode::Char('m'),
                    egui::Key::N => KeyCode::Char('n'),
                    egui::Key::O => KeyCode::Char('o'),
                    egui::Key::P => KeyCode::Char('p'),
                    egui::Key::Q => KeyCode::Char('q'),
                    egui::Key::R => KeyCode::Char('r'),
                    egui::Key::S => KeyCode::Char('s'),
                    egui::Key::T => KeyCode::Char('t'),
                    egui::Key::U => KeyCode::Char('u'),
                    egui::Key::V => KeyCode::Char('v'),
                    egui::Key::W => KeyCode::Char('w'),
                    egui::Key::X => KeyCode::Char('x'),
                    egui::Key::Y => KeyCode::Char('y'),
                    egui::Key::Z => KeyCode::Char('z'),
                    egui::Key::F1 => KeyCode::F(1),
                    egui::Key::F2 => KeyCode::F(2),
                    egui::Key::F3 => KeyCode::F(3),
                    egui::Key::F4 => KeyCode::F(4),
                    egui::Key::F5 => KeyCode::F(5),
                    egui::Key::F6 => KeyCode::F(6),
                    egui::Key::F7 => KeyCode::F(7),
                    egui::Key::F8 => KeyCode::F(8),
                    egui::Key::F9 => KeyCode::F(9),
                    egui::Key::F10 => KeyCode::F(10),
                    egui::Key::F11 => KeyCode::F(11),
                    egui::Key::F12 => KeyCode::F(12),
                    egui::Key::F13 => KeyCode::F(13),
                    egui::Key::F14 => KeyCode::F(14),
                    egui::Key::F15 => KeyCode::F(15),
                    egui::Key::F16 => KeyCode::F(16),
                    egui::Key::F17 => KeyCode::F(17),
                    egui::Key::F18 => KeyCode::F(18),
                    egui::Key::F19 => KeyCode::F(19),
                    egui::Key::F20 => KeyCode::F(20),
                    _ => return None,
                };
                let kind = if !pressed {
                    KeyEventKind::Release
                } else if repeat {
                    KeyEventKind::Repeat
                } else {
                    KeyEventKind::Press
                };
                let mut mods = KeyModifiers::NONE;
                if modifiers.shift {
                    mods |= KeyModifiers::SHIFT;
                }
                if modifiers.ctrl {
                    mods |= KeyModifiers::CONTROL;
                }
                if modifiers.alt {
                    mods |= KeyModifiers::ALT;
                }
                if modifiers.command {
                    mods |= KeyModifiers::SUPER;
                }
                Some(Event::KeyEvent(KeyEvent {
                    code,
                    modifiers: mods,
                    kind,
                    state: KeyEventState::NONE,
                }))
            }
            _ => None,
        }
    }
}

#[cfg(feature = "egui-desktop")]
impl BackendSuite<BackendType> for EguiBackendSuite {
    fn run(&mut self, mut terminal_app: impl TerminalApp<BackendType>) -> anyhow::Result<()> {
        use eframe::{AppCreator, egui};

        env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([400.0, 300.0])
                .with_min_inner_size([300.0, 220.0]),
            ..Default::default()
        };
        let backend = self.make_backend();
        let app_creator: AppCreator = Box::new(|cc| {
            terminal_app.init(backend).unwrap();
            let handler = EguiApplicationHandler::new(cc, terminal_app);
            Ok(Box::new(handler))
        });
        eframe::run_native("eframe template", native_options, app_creator)?;
        Ok(())
    }
}

#[cfg(feature = "egui-web")]
impl BackendSuite<BackendType> for EguiBackendSuite {
    fn run(
        &mut self,
        mut terminal_app: impl TerminalApp<BackendType> + 'static,
    ) -> anyhow::Result<()> {
        use crate::backend::egui::EguiApplicationHandler;
        use eframe::AppCreator;
        use eframe::wasm_bindgen::JsCast as _;

        // Redirect `log` message to `console.log` and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        let web_options = eframe::WebOptions::default();

        let backend = self.make_backend();
        wasm_bindgen_futures::spawn_local(async {
            let document = web_sys::window()
                .expect("No window")
                .document()
                .expect("No document");

            let canvas = document
                .get_element_by_id("the_canvas_id")
                .expect("Failed to find the_canvas_id")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("the_canvas_id was not a HtmlCanvasElement");

            let app_creator: AppCreator = Box::new(|cc| {
                terminal_app.init(backend).unwrap();
                let handler = EguiApplicationHandler::new(cc, terminal_app);
                Ok(Box::new(handler))
            });
            let start_result = eframe::WebRunner::new()
                .start(canvas, web_options, app_creator)
                .await;

            // Remove the loading text and spinner:
            if let Some(loading_text) = document.get_element_by_id("loading_text") {
                match start_result {
                    Ok(_) => {
                        loading_text.remove();
                    }
                    Err(e) => {
                        loading_text.set_inner_html(
                            "<p> The app has crashed. See the developer console for details. </p>",
                        );
                        panic!("Failed to start eframe: {e:?}");
                    }
                }
            }
        });

        Ok(())
    }
}

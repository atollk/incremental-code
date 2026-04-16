#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::backend::backend::{BackendSuite, TerminalApp};
use eframe::{CreationContext, Frame};
use egui_ratatui::RataguiBackend;
use soft_ratatui::embedded_graphics_unicodefonts::{mono_8x13_atlas, mono_8x13_bold_atlas, mono_8x13_italic_atlas};
use soft_ratatui::{EmbeddedGraphics, SoftBackend};
use std::sync::{LazyLock, Mutex};
use crate::backend::events::{Event, IntoEvent};

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
    pub fn new(
        _cc: &CreationContext<'_>,
        terminal_app: A,
    ) -> Self {
        EguiApplicationHandler {
            terminal_app,
        }
    }
}

impl<A: TerminalApp<BackendType>> eframe::App
for EguiApplicationHandler<A>
{
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        let events = ui.input(parse_input_events);
        self.terminal_app.frame(&events).unwrap();
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add(self.terminal_app.backend_mut());
        });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
    }
}

fn parse_input_events(input: &egui::InputState) -> Vec<Event> {
    input.events.iter().filter_map(IntoEvent::into_event).collect()
}

impl IntoEvent for egui::Event {
    fn into_event(self) -> Option<Event> {
        todo!()
    }
}

#[cfg(feature = "egui-desktop")]
impl BackendSuite<BackendType> for EguiBackendSuite {
    fn run(&mut self, mut terminal_app: impl TerminalApp<BackendType>) -> anyhow::Result<()> {
        use crate::backend::backend::TerminalApp;
        use crate::backend::egui::EguiApplicationHandler;
        use eframe::{egui, AppCreator};

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
    fn run(&mut self, mut terminal_app: impl TerminalApp<BackendType> + 'static) -> anyhow::Result<()> {
        use crate::backend::backend::TerminalApp;
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
                .start(
                    canvas,
                    web_options,
                    app_creator,
                )
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

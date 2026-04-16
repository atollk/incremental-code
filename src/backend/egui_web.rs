use crate::backend::backend::{BackendSuite, TerminalApp};
use eframe::{egui, AppCreator, CreationContext, Frame};
use egui_ratatui::RataguiBackend;
use soft_ratatui::embedded_graphics_unicodefonts::{
    mono_8x13_atlas, mono_8x13_bold_atlas, mono_8x13_italic_atlas,
};
use soft_ratatui::{EmbeddedGraphics, SoftBackend};
use std::sync::{LazyLock, Mutex};

pub type BackendType = RataguiBackend<EmbeddedGraphics>;

pub static BACKEND_INSTANCE: LazyLock<Mutex<EguiDesktopBackendSuite>> =
    LazyLock::new(|| Mutex::new(EguiDesktopBackendSuite {}));

pub struct EguiDesktopBackendSuite {}

impl EguiDesktopBackendSuite {
    fn make_backend(&self) -> BackendType {
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

impl BackendSuite<BackendType> for EguiDesktopBackendSuite {
    fn run(&mut self, mut terminal_app: impl TerminalApp<BackendType> + 'static) -> anyhow::Result<()> {
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
                let handler = EguiDesktopApplicationHandler::new(cc, terminal_app);
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

pub struct EguiDesktopApplicationHandler<A: TerminalApp<BackendType>> {
    terminal_app: A,
}

impl<A: TerminalApp<BackendType>> EguiDesktopApplicationHandler<A> {
    pub fn new(
        _cc: &CreationContext<'_>,
        terminal_app: A,
    ) -> Self {
        EguiDesktopApplicationHandler {
            terminal_app,
        }
    }
}

impl<A: TerminalApp<BackendType>> eframe::App
    for EguiDesktopApplicationHandler<A>
{
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        self.terminal_app.frame().unwrap();
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add(self.terminal_app.backend_mut());
        });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
    }
}

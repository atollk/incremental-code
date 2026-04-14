use eframe::egui;
use egui_ratatui::RataguiBackend;
use ratatui::Terminal;
use ratatui::prelude::Stylize;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use soft_ratatui::embedded_graphics_unicodefonts::{
    mono_8x13_atlas, mono_8x13_bold_atlas, mono_8x13_italic_atlas,
};
use soft_ratatui::{EmbeddedGraphics, SoftBackend};

struct MyEguiApp {
    // RataguiBackend<EmbeddedGraphics>, not RataguiBackend<SoftBackend<EmbeddedGraphics>>
    terminal: Terminal<RataguiBackend<EmbeddedGraphics>>,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let font_regular = mono_8x13_atlas();
        let font_italic = mono_8x13_italic_atlas();
        let font_bold = mono_8x13_bold_atlas();
        // Construction is still SoftBackend::<EmbeddedGraphics>::new(...)
        let soft_backend = SoftBackend::<EmbeddedGraphics>::new(
            100,
            50,
            font_regular,
            Some(font_bold),
            Some(font_italic),
        );
        let backend = RataguiBackend::new("soft_rat", soft_backend);
        let terminal = Terminal::new(backend).unwrap();
        Self { terminal }
    }

    fn example(&mut self) {}
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.terminal
            .draw(|frame| {
                let area = frame.area();
                let textik = format!("Hello eframe! The window area is {}", area);
                frame.render_widget(
                    Paragraph::new(textik)
                        .block(Block::new().title("Ratatui").borders(Borders::ALL))
                        .white()
                        .on_blue()
                        .wrap(Wrap { trim: false }),
                    area,
                );
            })
            .expect("epic fail");
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(self.terminal.backend_mut());
        });
    }
}

pub fn main() -> eframe::Result {
    Ok(())
}

mod web {
    use super::MyEguiApp;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[derive(Clone)]
    #[wasm_bindgen]
    pub struct WebHandle {
        runner: eframe::WebRunner,
    }

    #[wasm_bindgen]
    impl WebHandle {
        #[expect(clippy::new_without_default)]
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            eframe::WebLogger::init(log::LevelFilter::Debug).ok();
            Self {
                runner: eframe::WebRunner::new(),
            }
        }

        #[wasm_bindgen]
        pub async fn start(
            &self,
            canvas: web_sys::HtmlCanvasElement,
        ) -> Result<(), wasm_bindgen::JsValue> {
            self.runner
                .start(
                    canvas,
                    eframe::WebOptions::default(),
                    Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
                )
                .await
        }

        #[wasm_bindgen]
        pub fn destroy(&self) {
            self.runner.destroy();
        }

        #[wasm_bindgen]
        pub fn example(&self) {
            if let Some(mut app) = self.runner.app_mut::<MyEguiApp>() {
                app.example();
            }
        }

        #[wasm_bindgen]
        pub fn has_panicked(&self) -> bool {
            self.runner.has_panicked()
        }

        #[wasm_bindgen]
        pub fn panic_message(&self) -> Option<String> {
            self.runner.panic_summary().map(|s| s.message())
        }

        #[wasm_bindgen]
        pub fn panic_callstack(&self) -> Option<String> {
            self.runner.panic_summary().map(|s| s.callstack())
        }
    }
}
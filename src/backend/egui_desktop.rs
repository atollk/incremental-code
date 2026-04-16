#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

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
    fn run(&mut self, mut terminal_app: impl TerminalApp<BackendType>) -> anyhow::Result<()> {
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
            let handler = EguiDesktopApplicationHandler::new(cc, self, terminal_app);
            Ok(Box::new(handler))
        });
        eframe::run_native("eframe template", native_options, app_creator)?;
        Ok(())
    }
}

pub struct EguiDesktopApplicationHandler<'a, A: TerminalApp<BackendType>> {
    backend_suite: &'a EguiDesktopBackendSuite,
    terminal_app: A,
}

impl<'a, A: TerminalApp<BackendType>> EguiDesktopApplicationHandler<'a, A> {
    /// Called once before the first frame.
    pub fn new(
        _cc: &CreationContext<'_>,
        backend_suite: &'a EguiDesktopBackendSuite,
        terminal_app: A,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        EguiDesktopApplicationHandler {
            backend_suite,
            terminal_app,
        }
    }
}

impl<A: TerminalApp<BackendType>> eframe::App
    for EguiDesktopApplicationHandler<'_, A>
{
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut Frame) {
        self.terminal_app.frame().unwrap();
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        todo!()
    }
}

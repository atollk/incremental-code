#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::backend::backend::BackendSuite;
use eframe::{AppCreator, CreationContext, Frame, egui};
use egui::Context;
use egui_ratatui::RataguiBackend;
use ratatui::Terminal;
use ratatui::prelude::Stylize;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use soft_ratatui::embedded_graphics_unicodefonts::{
    mono_8x13_atlas, mono_8x13_bold_atlas, mono_8x13_italic_atlas,
};
use soft_ratatui::{EmbeddedGraphics, SoftBackend};
use std::rc::Rc;

pub struct EguiDesktopBackendSuite {}

impl EguiDesktopBackendSuite {
    fn make_backend(&self) -> RataguiBackend<EmbeddedGraphics> {
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
        let backend = RataguiBackend::new("soft_rat", soft_backend);
        backend
    }
}

impl BackendSuite<RataguiBackend<EmbeddedGraphics>> for EguiDesktopBackendSuite {
    fn run(&mut self, runner: impl FnMut(RataguiBackend<EmbeddedGraphics>)) -> anyhow::Result<()> {
        env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([400.0, 300.0])
                .with_min_inner_size([300.0, 220.0]),
            ..Default::default()
        };
        let app_creator = Box::new(|cc| {
            Ok(Box::new(EguiDesktopApplicationHandler::new(
                cc, self, runner,
            )))
        });
        eframe::run_native("eframe template", native_options, app_creator)?;
        Ok(())
    }
}

pub struct EguiDesktopApplicationHandler<'a, F: FnMut(RataguiBackend<EmbeddedGraphics>)> {
    backend_suite: &'a EguiDesktopBackendSuite,
    runner: F,
}

impl<'a> EguiDesktopApplicationHandler<'a> {
    /// Called once before the first frame.
    pub fn new(
        _cc: &CreationContext<'_>,
        backend_suite: &'a EguiDesktopBackendSuite,
        runner: impl FnMut(RataguiBackend<EmbeddedGraphics>),
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        EguiDesktopApplicationHandler {
            backend_suite,
            runner,
        }
    }
}

impl<F: FnMut(RataguiBackend<EmbeddedGraphics>)> eframe::App for EguiDesktopApplicationHandler<'_, F> {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        todo!()
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.label("this is a label");

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

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add(self.terminal.backend_mut()); // <-- uncomment this
        });
    }

    fn update(&mut self, _ctx: &Context, _frame: &mut Frame) {}
}

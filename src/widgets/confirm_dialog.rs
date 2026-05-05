use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind};
use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::home_terminal::HomeTerminalScene;
use crate::game_state::{with_game_state, with_game_state_mut};
use crate::widgets::hud::draw_hud;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::terminal::Frame;
use ratatui_core::widgets::Widget;

#[derive(Copy, Clone)]
/// The outcome returned by a [`ConfirmDialog`] after the player responds.
pub enum ConfirmResult {
    Yes,
    No,
    Cancel,
}

/// A modal yes/no/cancel confirmation dialog.
///
/// Render it with `frame.render_widget(&dialog, area)` and call
/// [`handle_event`](Self::handle_event) each frame to process keyboard input.
pub struct ConfirmDialog {
    title: String,
    message: String,
    result: Option<ConfirmResult>,
}

impl ConfirmDialog {
    /// Creates a new dialog with the given title and message text.
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            result: None,
        }
    }

    /// Processes an input event, updating the internal result if the player presses y/n/Esc.
    pub fn handle_event(&mut self, event: &Event) {
        if let Event::KeyEvent(key) = event {
            if key.kind == KeyEventKind::Release {
                return;
            }
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.result = Some(ConfirmResult::Yes);
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.result = Some(ConfirmResult::No);
                }
                KeyCode::Esc => {
                    self.result = Some(ConfirmResult::Cancel);
                }
                _ => {}
            }
        }
    }

    /// Returns the player's choice, or `None` if the dialog is still waiting for input.
    pub fn result(&self) -> Option<ConfirmResult> {
        self.result
    }

    pub fn render_overlay(
        dialog: &mut Option<ConfirmDialog>,
        frame: &mut Frame,
        events: &[Event],
        update_underlay: impl FnOnce() -> SceneSwitch,
        render_content: impl FnOnce(&mut Frame),
        on_result: impl FnOnce(ConfirmResult) -> SceneSwitch,
    ) -> SceneSwitch {
        if let Some(dialog) = dialog {
            for event in events {
                dialog.handle_event(event);
            }
            let dialog_scene_switch = match dialog.result() {
                Some(result) => on_result(result.clone()),
                None => SceneSwitch::NoSwitch,
            };

            render_content(frame);

            frame.render_widget(&*dialog, frame.area());
            dialog_scene_switch
        } else {
            let scene_switch = update_underlay();
            render_content(frame);
            scene_switch
        }
    }
}

impl Widget for &ConfirmDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .split(area);

        let horizontal = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(42),
            Constraint::Fill(1),
        ])
        .split(vertical[1]);

        let dialog_area = horizontal[1];

        Clear.render(dialog_area, buf);

        let block = Block::new()
            .borders(Borders::ALL)
            .title(self.title.as_str());
        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        Paragraph::new(self.message.as_str())
            .centered()
            .render(inner, buf);
    }
}

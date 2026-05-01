use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::widgets::Widget;

#[derive(Copy, Clone)]
pub enum ConfirmResult {
    Yes,
    No,
    Cancel,
}

pub struct ConfirmDialog {
    title: String,
    message: String,
    result: Option<ConfirmResult>,
}

impl ConfirmDialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            result: None,
        }
    }

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

    pub fn result(&self) -> Option<ConfirmResult> {
        self.result
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

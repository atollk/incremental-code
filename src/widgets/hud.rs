use crate::game_state::with_game_state;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::terminal::Frame;
use ratatui_core::widgets::Widget;

/// Fixed width (in terminal columns) reserved for the HUD panel.
pub const HUD_WIDTH: u16 = 22;

/// Renders the HUD panel showing the player's current resource totals.
pub struct HudWidget;

impl Widget for HudWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let resources = with_game_state(|s| format!("{}", s.total_resources().fmt_multiline()));

        let block = Block::new().borders(Borders::ALL).title(" HUD ");
        let inner = block.inner(area);
        block.render(area, buf);

        Paragraph::new(resources)
            .wrap(Wrap { trim: false })
            .render(inner, buf);
    }
}

/// Renders the [`HudWidget`] on the left side of the frame and returns the remaining content area.
///
/// If the frame is narrower than [`HUD_WIDTH`], the full area is returned unchanged.
pub fn hud_layout(frame: &mut Frame) -> Rect {
    let full_area = frame.area();
    if full_area.width <= HUD_WIDTH {
        return full_area;
    }
    let [hud_area, content_area] =
        Layout::horizontal([Constraint::Length(HUD_WIDTH), Constraint::Fill(1)]).areas(full_area);
    frame.render_widget(HudWidget, hud_area);
    content_area
}

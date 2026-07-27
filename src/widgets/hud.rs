use crate::game_scenes::logic::auto_run::with_auto_run;
use crate::game_scenes::upgrades::count_buyable;
use crate::game_state::{with_auto_saver_mut, with_game_state};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::terminal::Frame;
use ratatui_core::text::Text;
use ratatui_core::widgets::Widget;
use ratatui_widgets::gauge::LineGauge;

/// Fixed width (in terminal columns) reserved for the HUD panel.
pub const HUD_WIDTH: u16 = 22;

/// Renders the HUD panel showing the player's current resource totals.
pub struct HudWidget;

impl Widget for HudWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut text = Text::default();

        let resource_string = with_game_state(|s| s.total_resources().fmt_multiline().to_string());
        resource_string
            .lines()
            .for_each(|line| text.push_line(line));
        text.push_line("");

        let buyable = with_game_state(|s| {
            let resources = s.total_resources();
            count_buyable(&s.upgrades, &resources)
        });
        text.push_line(format!("Buyable upgrades: {buyable}"));
        text.push_line("");

        let time_since_last_save = with_auto_saver_mut(|auto_saver| auto_saver.since_last_save());
        text.push_line(format!(
            "Time since last save: {}s",
            time_since_last_save.as_secs()
        ));
        text.push_line("");

        let block = Block::new().borders(Borders::ALL).title(" HUD ");
        let inner = block.inner(area);
        block.render(area, buf);

        let autorun_progress = with_auto_run(|ar| ar.get_progress());
        let text_height = text.height() as u16;

        match autorun_progress {
            Some(ratio) => {
                let [text_area, gauge_area] =
                    Layout::vertical([Constraint::Length(text_height), Constraint::Length(1)])
                        .areas(inner);
                Paragraph::new(text)
                    .wrap(Wrap { trim: false })
                    .render(text_area, buf);
                LineGauge::default()
                    .ratio(ratio)
                    .label("")
                    .render(gauge_area, buf);
            }
            None => {
                Paragraph::new(text)
                    .wrap(Wrap { trim: false })
                    .render(inner, buf);
            }
        }
    }
}

/// Renders the [`HudWidget`] on the left side of the frame and returns the remaining content area.
///
/// If the frame is narrower than [`HUD_WIDTH`], the full area is returned unchanged.
pub fn draw_hud(frame: &mut Frame) -> Rect {
    let full_area = frame.area();
    if full_area.width <= HUD_WIDTH {
        return full_area;
    }
    let [hud_area, content_area] =
        Layout::horizontal([Constraint::Length(HUD_WIDTH), Constraint::Fill(1)]).areas(full_area);
    frame.render_widget(HudWidget, hud_area);
    content_area
}

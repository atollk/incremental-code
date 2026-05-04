use crate::game_scenes::base::SceneSwitch;
use crate::game_state::save_game_state;
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use ratatui_core::style::{Color, Style};
use ratatui_core::text::Text;
use ratatui_widgets::paragraph::Paragraph;

pub(super) fn save_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    match save_game_state() {
        Ok(_) => {
            let text = "Game Saved";
            Box::new(ParagraphCmd::new(Paragraph::new(Text::raw(text))))
        }
        Err(e) => {
            let text = format!("Error - could not save the game: {e}");
            Box::new(ParagraphCmd::new(Paragraph::new(Text::styled(
                text,
                Style::default().fg(Color::Red),
            ))))
        }
    }
}

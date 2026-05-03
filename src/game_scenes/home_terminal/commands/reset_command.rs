use crate::game_scenes::base::SceneSwitch;
use crate::game_state::erase_game_state;
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use ratatui_core::text::Text;
use ratatui_widgets::paragraph::Paragraph;

pub(super) fn reset_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    let text = match erase_game_state() {
        Ok(()) => Text::raw("Save state erased successfully."),
        Err(err) => Text::raw(format!("Could not erase state: {}", err)),
    };
    Box::new(ParagraphCmd::new(Paragraph::new(text)))
}

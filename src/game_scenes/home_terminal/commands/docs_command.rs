use crate::game_scenes::base::SceneSwitch;
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use ratatui_core::text::Text;
use ratatui_widgets::paragraph::Paragraph;

pub(super) fn docs_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    let text = Text::from("todo");
    Box::new(ParagraphCmd::new(Paragraph::new(text)))
}

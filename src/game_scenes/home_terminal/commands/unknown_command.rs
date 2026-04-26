use crate::game_scenes::base::SceneSwitch;
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use ratatui_core::text::Text;
use ratatui_widgets::paragraph::Paragraph;

pub fn unknown_cmd(cmd: String) -> Box<dyn RunningCommand<SceneSwitch>> {
    let text = format!("Unknown command '{cmd}'. For a list of available commands, try 'help'.");
    let text = Text::raw(text);
    Box::new(ParagraphCmd::new(Paragraph::new(text)))
}

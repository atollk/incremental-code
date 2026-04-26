use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::home_terminal::command_list;
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use ratatui_core::text::{Line, Text};
use ratatui_widgets::paragraph::Paragraph;

pub(super) fn help_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    let available_commands = command_list();
    let lines = std::iter::once("List of available commands:".to_string())
        .chain(
            available_commands
                .iter()
                .map(|c| format!("  {}\t - {}", c.name, c.help_description)),
        )
        .map(Line::from)
        .collect::<Vec<_>>();
    let text = Text::from(lines);
    Box::new(ParagraphCmd::new(Paragraph::new(text)))
}

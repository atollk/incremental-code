use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::home_terminal::command_list;
use crate::game_scenes::home_terminal::commands::CommandListItem;
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use ratatui_core::text::{Line, Text};
use ratatui_widgets::paragraph::Paragraph;

pub(super) fn help_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    let available_commands = command_list();
    let lines = std::iter::once(Line::from("List of available commands:"))
        .chain(available_commands.into_iter().map(|c| match c {
            CommandListItem::Command(c) => {
                Line::from(format!("  {}\t - {}", c.name, c.help_description))
            }
            CommandListItem::Separator => Line::from("\n"),
        }))
        .collect::<Vec<_>>();
    let text = Text::from(lines);
    Box::new(ParagraphCmd::new(Paragraph::new(text)))
}

use crate::backend::events::Event;
use crate::game_scenes::base::Scene;
use crate::game_scenes::base::SceneSwitch;
use crate::widgets::terminal::{RunningCommand, TerminalWidget};
use ratatui::widgets::Paragraph;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::terminal::Frame;
use ratatui_core::text::{Line, Text};
use ratatui_core::widgets::Widget;
use std::time::Duration;

pub struct HomeTerminalScene {
    terminal_widget: TerminalWidget,
}

impl HomeTerminalScene {
    pub fn new() -> Self {
        HomeTerminalScene {
            terminal_widget: TerminalWidget::new(),
        }
    }

    fn handle_terminal_command(&self, cmd: &str) -> Box<dyn RunningCommand> {
        match cmd.trim() {
            "help" => Box::new(help_cmd()),
            _ => Box::new(unknown_cmd(cmd.to_owned())),
        }
    }
}

impl Scene for HomeTerminalScene {
    fn frame(
        &mut self,
        events: &[Event],
        frame: &mut Frame,
        time_delta: web_time::Duration,
    ) -> SceneSwitch {
        let cmd = self.terminal_widget.update(events, time_delta);
        if let Some(cmd) = cmd {
            self.terminal_widget
                .set_running(&cmd, self.handle_terminal_command(&cmd));
        }
        frame.render_widget(&self.terminal_widget, frame.area());
        SceneSwitch::NoSwitch
    }
}

fn unknown_cmd(cmd: String) -> impl RunningCommand {
    let text = format!("Unknown command '{cmd}'. For a list of available commands, try 'help'.");
    let text = Text::raw(text);
    ParagraphCmd {
        paragraph: Paragraph::new(text),
    }
}

fn help_cmd() -> impl RunningCommand {
    let available_commands = vec![
        "help\t - Displays this help text",
    ];
    let lines = std::iter::once("List of available commands:").chain(available_commands).map(Line::from).collect::<Vec<_>>();
    let text = Text::from(lines);
    ParagraphCmd {
        paragraph: Paragraph::new(text),
    }
}

/// Shows a paragraph of text and finishes immediately.
struct ParagraphCmd<'a> {
    paragraph: Paragraph<'a>,
}

impl RunningCommand for ParagraphCmd<'_> {
    fn is_done(&self) -> bool {
        true
    }

    fn update(&mut self, _events: &[Event], _time_delta: Duration) {}

    fn render(&self, area: Rect, buf: &mut Buffer) {
        (&self.paragraph).render(area, buf);
    }

    fn height(&self, columns: u16) -> u16 {
        self.paragraph.line_count(columns) as u16
    }
}

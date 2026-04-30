use crate::backend::events::Event;
use crate::backend::input::KeyCode;
use crate::game_scenes::base::Scene;
use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::home_terminal::commands::{command_list, unknown_cmd};
use crate::widgets::hud::hud_layout;
use crate::widgets::terminal::{RunningCommand, TerminalWidget};
use itertools::Itertools;
use ratatui_core::terminal::Frame;
use web_time::Duration;

pub struct HomeTerminalScene {
    terminal_widget: TerminalWidget<SceneSwitch>,
}

impl Default for HomeTerminalScene {
    fn default() -> Self {
        Self::new()
    }
}

impl HomeTerminalScene {
    pub fn new() -> Self {
        HomeTerminalScene {
            terminal_widget: TerminalWidget::new(),
        }
    }

    fn handle_terminal_command(&self, cmd: &str) -> Box<dyn RunningCommand<SceneSwitch>> {
        let commands = command_list();
        if let Ok(cmd) = commands.iter().filter(|c| c.name == cmd).exactly_one() {
            (cmd.runner)()
        } else {
            unknown_cmd(cmd.to_owned())
        }
    }
}

impl Scene for HomeTerminalScene {
    fn frame(&mut self, events: &[Event], frame: &mut Frame, time_delta: Duration) -> SceneSwitch {
        // Execute command
        let cmd = self.terminal_widget.update(events, time_delta);
        if let Some(cmd) = cmd {
            self.terminal_widget
                .set_running(&cmd, self.handle_terminal_command(&cmd));
        }

        // Autocomplete
        for event in events {
            if let Event::KeyEvent(key) = event
                && key.code == KeyCode::Tab
            {
                if self.terminal_widget.input.is_empty() {
                    continue;
                }
                let completion_candidates = command_list()
                    .iter()
                    .map(|cmd| cmd.name)
                    .filter(|cmd| cmd.starts_with(&self.terminal_widget.input))
                    .collect_vec();
                match completion_candidates.len() {
                    0 => {}
                    1 => {
                        self.terminal_widget.input = completion_candidates[0].to_owned();
                    }
                    _ => {
                        todo!()
                    }
                }
            }
        }

        // Draw widget
        let content_area = hud_layout(frame);
        frame.render_widget(&self.terminal_widget, content_area);

        // Switch scene
        if let Some(cmd) = &self.terminal_widget.running {
            cmd.get_metadata()
        } else {
            SceneSwitch::NoSwitch
        }
    }
}

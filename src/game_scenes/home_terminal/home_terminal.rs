use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind, KeyEventState};
use crate::game_scenes::base::Scene;
use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::home_terminal::commands::{command_list, unknown_cmd};
use crate::game_state::with_game_state;
use crate::widgets::hud::draw_hud;
use crate::widgets::terminal::{RunningCommand, TerminalWidget};
use itertools::Itertools;
use ratatui_core::terminal::Frame;
use web_time::Duration;

pub struct HomeTerminalScene {
    terminal_widget: TerminalWidget<SceneSwitch>,
    autocomplete_cycle: Option<(String, usize)>,
    pub(crate) height: u16,
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
            autocomplete_cycle: None,
            height: 0,
        }
    }

    fn handle_terminal_command(&self, cmd: &str) -> Box<dyn RunningCommand<SceneSwitch>> {
        let commands = command_list();
        if let Ok(cmd) = commands.iter().filter(|c| c.name == cmd).exactly_one() {
            (cmd.runner)(self)
        } else {
            unknown_cmd(cmd.to_owned())
        }
    }

    fn autocomplete(&mut self) {
        if self.terminal_widget.input.is_empty() {
            return;
        }
        let (input, cycle) = self
            .autocomplete_cycle
            .get_or_insert_with(|| (self.terminal_widget.input.clone(), 0));
        let completion_candidates = command_list()
            .iter()
            .map(|cmd| cmd.name)
            .filter(|cmd| cmd.starts_with(&*input))
            .collect_vec();
        if completion_candidates.len() == 0 {
            return;
        }
        self.terminal_widget.input = completion_candidates[*cycle].to_owned();
        *cycle = (*cycle + 1) % completion_candidates.len();
        self.terminal_widget.input_cursor = self.terminal_widget.input.len();
    }
}

impl Scene for HomeTerminalScene {
    fn frame(&mut self, events: &[Event], frame: &mut Frame, time_delta: Duration) -> SceneSwitch {
        self.height = frame.area().height;

        // Execute command
        let cmd = self.terminal_widget.update(events, time_delta);
        if let Some(cmd) = cmd {
            self.terminal_widget
                .set_running(&cmd, self.handle_terminal_command(&cmd));
        }

        // Autocomplete
        for event in events {
            if let Event::KeyEvent(key) = event
                && key.kind == KeyEventKind::Press
            {
                if key.code == KeyCode::Tab {
                    self.autocomplete();
                } else {
                    self.autocomplete_cycle = None;
                }
            }
        }

        // Draw HUD
        let content_area = if with_game_state(|game_state| game_state.upgrades.unlock_hud.value()) {
            draw_hud(frame)
        } else {
            frame.area()
        };
        frame.render_widget(&self.terminal_widget, content_area);

        // Switch scene
        if let Some(cmd) = &self.terminal_widget.running {
            cmd.get_metadata()
        } else {
            SceneSwitch::NoSwitch
        }
    }
}

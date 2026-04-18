use crate::backend::events::Event;
use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::base::Scene;
use crate::widgets::terminal::{RunningCommand, TerminalWidget};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::terminal::Frame;
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
        Box::new(DummyCmd{})
    }
}

struct DummyCmd {}

impl RunningCommand for DummyCmd {
    fn is_done(&self) -> bool {
        true
    }

    fn update(&mut self, events: &[Event], time_delta: Duration) {

    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let mut text = ratatui::text::Text::from("");
        text.render(area, buf);
    }

    fn height(&self) -> u16 {
        1
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
            self.terminal_widget.running = Some(self.handle_terminal_command(&cmd));
        }
        frame.render_widget(&self.terminal_widget, frame.area());
        SceneSwitch::NoSwitch
    }
}

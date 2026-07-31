use crate::backend::events::Event;
use crate::game_scenes::base::{Scene, SceneSwitch};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui_core::terminal::Frame;
use web_time::Duration;

const MESSAGE_LINES: &[&str] = &[
    "Congratulations!",
    "",
    "You have completed the game.",
    "",
    "Thank you for playing :)",
];

pub struct VictoryScene;

impl VictoryScene {
    pub fn new() -> Self {
        VictoryScene
    }
}

impl Default for VictoryScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for VictoryScene {
    fn frame(
        &mut self,
        _events: &[Event],
        frame: &mut Frame,
        _time_delta: Duration,
    ) -> SceneSwitch {
        let area = frame.area();
        let lines_len = MESSAGE_LINES.len() as u16;
        let [_, center, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(lines_len),
            Constraint::Fill(1),
        ])
        .areas(area);

        let text: Vec<Line> = MESSAGE_LINES
            .iter()
            .map(|line| Line::raw(*line).alignment(Alignment::Center))
            .collect();

        frame.render_widget(
            Paragraph::new(text).style(Style::default().fg(Color::White)),
            center,
        );

        SceneSwitch::NoSwitch
    }
}

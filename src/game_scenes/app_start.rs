use crate::backend::events::Event;
use crate::game_scenes::base::{Scene, SceneSwitch};
use crate::game_scenes::home_terminal::HomeTerminalScene;
use crate::game_scenes::reboot::RebootScene;
use crate::game_scenes::victory::VictoryScene;
use crate::game_state::{load_game_state, load_settings, with_auto_saver_mut, with_game_state_mut};
use ratatui_core::terminal::Frame;
use web_time::Duration;

pub struct AppStartScene;

impl AppStartScene {
    pub fn new() -> Self {
        AppStartScene
    }

    fn finish(&self) -> SceneSwitch {
        let loaded_state = load_game_state().unwrap_or_else(|e| {
            log::error!("{e}");
            false
        });
        if let Err(e) = load_settings() {
            log::error!("{e}");
        }
        let won = with_game_state_mut(|game_state| game_state.upgrades.win_condition.value());
        with_auto_saver_mut(|auto_saver| auto_saver.start(Duration::from_secs(60)));
        let scene: Box<dyn Scene> = if won {
            Box::new(VictoryScene::new())
        } else if loaded_state {
            Box::new(HomeTerminalScene::new())
        } else {
            Box::new(RebootScene::new(true, 20))
        };
        SceneSwitch::SwitchTo(scene)
    }

    #[cfg(not(feature = "ratzilla"))]
    fn frame_impl(&mut self, _events: &[Event], _frame: &mut Frame) -> SceneSwitch {
        self.finish()
    }

    #[cfg(feature = "ratzilla")]
    fn frame_impl(&mut self, events: &[Event], frame: &mut Frame) -> SceneSwitch {
        use crate::backend::input::{KeyEventKind, MouseEventKind};

        for event in events {
            let is_interaction = match event {
                Event::KeyEvent(e) => e.kind == KeyEventKind::Press,
                Event::MouseEvent(e) => matches!(e.kind, MouseEventKind::Down(_)),
                Event::MetaEvent(_) => false,
            };
            if is_interaction {
                return self.finish();
            }
        }

        use ratatui::layout::{Alignment, Constraint, Layout};
        use ratatui::style::{Color, Style};
        use ratatui::text::Line;
        use ratatui::widgets::Paragraph;

        let area = frame.area();
        let [_, center, _] = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(area);

        frame.render_widget(
            Paragraph::new(Line::raw("click to start"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White)),
            center,
        );

        SceneSwitch::NoSwitch
    }
}

impl Default for AppStartScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for AppStartScene {
    fn frame(&mut self, events: &[Event], frame: &mut Frame, _time_delta: Duration) -> SceneSwitch {
        self.frame_impl(events, frame)
    }
}

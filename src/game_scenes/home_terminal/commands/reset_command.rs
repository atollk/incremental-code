use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind};
use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::reboot::RebootScene;
use crate::game_state::erase_game_state;
use crate::widgets::terminal::RunningCommand;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use std::time::Duration;

enum ResetState {
    Asking,
    Confirmed,
    Cancelled,
}

struct ResetCmd {
    state: ResetState,
}

pub(super) fn reset_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    Box::new(ResetCmd {
        state: ResetState::Asking,
    })
}

impl RunningCommand<SceneSwitch> for ResetCmd {
    fn is_done(&self) -> bool {
        matches!(self.state, ResetState::Cancelled)
    }

    fn update(&mut self, events: &[Event], _time_delta: Duration) {
        if !matches!(self.state, ResetState::Asking) {
            return;
        }
        for event in events {
            let Event::KeyEvent(key) = event else {
                continue;
            };
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Err(e) = erase_game_state() {
                        log::error!("reset: {e}");
                    }
                    self.state = ResetState::Confirmed;
                    return;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Enter => {
                    self.state = ResetState::Cancelled;
                    return;
                }
                _ => {}
            }
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let text = match self.state {
            ResetState::Asking => "Reset all progress? [y/N]",
            ResetState::Confirmed => "Rebooting...",
            ResetState::Cancelled => "Reset cancelled.",
        };
        buf.set_string(area.x, area.y, text, Style::default());
    }

    fn height(&self, _columns: u16) -> u16 {
        1
    }

    fn get_metadata(&self) -> SceneSwitch {
        if matches!(self.state, ResetState::Confirmed) {
            SceneSwitch::SwitchTo(Box::new(RebootScene::new()))
        } else {
            SceneSwitch::NoSwitch
        }
    }
}

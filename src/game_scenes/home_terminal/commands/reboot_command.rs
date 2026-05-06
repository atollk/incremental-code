use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind};
use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::reboot::RebootScene;
use crate::widgets::terminal::RunningCommand;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use std::time::Duration;

enum RebootState {
    Asking,
    Confirmed,
    Cancelled,
}

struct RebootCmd {
    state: RebootState,
}

pub(super) fn reboot_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    Box::new(RebootCmd {
        state: RebootState::Asking,
    })
}

impl RunningCommand<SceneSwitch> for RebootCmd {
    fn is_done(&self) -> bool {
        matches!(self.state, RebootState::Cancelled)
    }

    fn update(&mut self, events: &[Event], _time_delta: Duration) {
        if !matches!(self.state, RebootState::Asking) {
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
                    // TODO: carry over currency
                    self.state = RebootState::Confirmed;
                    return;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Enter => {
                    self.state = RebootState::Cancelled;
                    return;
                }
                _ => {}
            }
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let text = match self.state {
            RebootState::Asking => {
                "Reboot? This will reset all upgrades but give additional resources. [y/N]"
            }
            RebootState::Confirmed => "Rebooting...",
            RebootState::Cancelled => "Reboot cancelled.",
        };
        buf.set_string(area.x, area.y, text, Style::default());
    }

    fn height(&self, _columns: u16) -> u16 {
        1
    }

    fn get_metadata(&self) -> SceneSwitch {
        if matches!(self.state, RebootState::Confirmed) {
            SceneSwitch::SwitchTo(Box::new(RebootScene::new(false, 40)))
        } else {
            SceneSwitch::NoSwitch
        }
    }
}

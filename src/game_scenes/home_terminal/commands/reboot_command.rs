use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind};
use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::reboot::RebootScene;
use crate::game_state::{with_game_state, with_game_state_mut};
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::text::Text;
use ratatui_core::widgets::Widget;
use ratatui_widgets::paragraph::Paragraph;
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
    with_game_state_mut(|game_state| -> Box<dyn RunningCommand<SceneSwitch>> {
        if game_state.upgrades.additive_reboot.value().0 {
            let resource_gain = game_state.prestige_currency().1;
            let text = format!("Gained {} from reboot.", resource_gain.fmt_oneline());
            game_state.current_resources += resource_gain;
            Box::new(ParagraphCmd::new(Paragraph::new(Text::from(text))))
        } else {
            Box::new(RebootCmd {
                state: RebootState::Asking,
            })
        }
    })
}

impl RebootCmd {
    fn current_text(&self) -> String {
        match self.state {
            RebootState::Asking => {
                let resource_gain = with_game_state(|game_state| game_state.prestige_currency()).1;
                format!(
                    "This will reset all upgrades but give additional resources.\nWith your current resources, you will restart with {}\nReboot?  [y/N]",
                    resource_gain.fmt_oneline()
                )
            }
            RebootState::Confirmed => "Rebooting...".to_string(),
            RebootState::Cancelled => "Reboot cancelled.".to_string(),
        }
    }
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
                    with_game_state_mut(|game_state| game_state.prestige());
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
        let text = self.current_text();
        let paragraph = Paragraph::new(text);
        paragraph.render(area, buf);
    }

    fn height(&self, _columns: u16) -> u16 {
        self.current_text().lines().count() as u16
    }

    fn get_metadata(&self) -> SceneSwitch {
        if matches!(self.state, RebootState::Confirmed) {
            SceneSwitch::SwitchTo(Box::new(RebootScene::new(false, 40)))
        } else {
            SceneSwitch::NoSwitch
        }
    }
}

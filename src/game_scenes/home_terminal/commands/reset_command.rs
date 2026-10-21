use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind};
use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::reboot::RebootScene;
use crate::game_state::{GameState, erase_game_state, with_game_state_mut};
use crate::widgets::terminal::RunningCommand;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use std::time::Duration;

/// Seconds the second confirmation prompt ignores input, to prevent accidental resets.
const FINAL_CONFIRM_DELAY: Duration = Duration::from_secs(5);

enum ResetState {
    Asking,
    /// Second confirmation; `y` is accepted only once `elapsed` reaches [`FINAL_CONFIRM_DELAY`].
    AskingFinal {
        elapsed: Duration,
    },
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

    fn update(&mut self, events: &[Event], time_delta: Duration) {
        let final_ready = match &mut self.state {
            ResetState::Asking => false,
            ResetState::AskingFinal { elapsed } => {
                *elapsed += time_delta;
                *elapsed >= FINAL_CONFIRM_DELAY
            }
            _ => return,
        };
        for event in events {
            let Event::KeyEvent(key) = event else {
                continue;
            };
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if matches!(self.state, ResetState::Asking) {
                        self.state = ResetState::AskingFinal {
                            elapsed: Duration::ZERO,
                        };
                        return;
                    }
                    if !final_ready {
                        continue;
                    }
                    with_game_state_mut(|game_state| {
                        *game_state = GameState::default();
                        game_state.on_upgrades_commit();
                    });
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
        let text = match &self.state {
            ResetState::Asking => "Reset all progress? [y/N]".to_string(),
            ResetState::AskingFinal { elapsed } => {
                if *elapsed >= FINAL_CONFIRM_DELAY {
                    "This will delete all progress. Really reset? [y/N]".to_string()
                } else {
                    let left = (FINAL_CONFIRM_DELAY - *elapsed).as_secs() + 1;
                    format!(
                        "This will delete ALL progress and your save files. It cannot be undone. Really reset? [y/N] (wait {left}s)"
                    )
                }
            }
            ResetState::Confirmed => "Rebooting...".to_string(),
            ResetState::Cancelled => "Reset cancelled.".to_string(),
        };
        buf.set_string(area.x, area.y, &text, Style::default());
    }

    fn height(&self, _columns: u16) -> u16 {
        1
    }

    fn get_metadata(&self) -> SceneSwitch {
        if matches!(self.state, ResetState::Confirmed) {
            SceneSwitch::SwitchTo(Box::new(RebootScene::new(false, 40)))
        } else {
            SceneSwitch::NoSwitch
        }
    }
}

use crate::backend::audio::with_audio_backend;
use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind};
use crate::game_scenes::base::SceneSwitch;
use crate::widgets::terminal::RunningCommand;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use std::time::Duration;

struct VolumeCmd {
    original: u8,
    current: u8,
    done: bool,
    cancelled: bool,
}

pub(super) fn volume_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    let initial = with_audio_backend(|audio| audio.get_volume())
        .map(|v| (v * 100.) as u8)
        .unwrap_or(100);
    Box::new(VolumeCmd {
        original: initial,
        current: initial,
        done: false,
        cancelled: false,
    })
}

impl VolumeCmd {
    fn bar_text(&self) -> String {
        let filled = (self.current as usize * 20 / 100).min(20);
        let empty = 20 - filled;
        format!(
            "Volume: [{filled}{empty}] {pct}%",
            filled = "█".repeat(filled),
            empty = "░".repeat(empty),
            pct = self.current,
        )
    }

    fn sync_volume(&self) {
        with_audio_backend(|audio| audio.set_volume(self.current as f32 / 100.));
    }
}

impl RunningCommand<SceneSwitch> for VolumeCmd {
    fn is_done(&self) -> bool {
        self.done
    }

    fn update(&mut self, events: &[Event], _time_delta: Duration) {
        if self.done {
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
                KeyCode::Left => {
                    self.current = self.current.saturating_sub(1);
                    self.sync_volume();
                }
                KeyCode::Right => {
                    self.current = self.current.saturating_add(1).min(100);
                    self.sync_volume();
                }
                KeyCode::Enter => {
                    self.done = true;
                    return;
                }
                KeyCode::Esc => {
                    self.current = self.original;
                    self.done = true;
                    self.cancelled = true;
                    self.sync_volume();
                    return;
                }
                _ => {}
            }
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let text: String = if !self.done {
            self.bar_text()
        } else if self.cancelled {
            format!("Volume restored to {}%.", self.original)
        } else {
            format!("Volume set to {}%.", self.current)
        };
        buf.set_string(area.x, area.y, &text, Style::default());
    }

    fn height(&self, _columns: u16) -> u16 {
        1
    }

    fn get_metadata(&self) -> SceneSwitch {
        SceneSwitch::NoSwitch
    }
}

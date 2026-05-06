use crate::backend::events::Event;
use crate::game_scenes::base::{Scene, SceneSwitch};
use crate::game_scenes::home_terminal::HomeTerminalScene;
use crate::widgets::cctv_animation::{CctvAnimation, CctvAnimationState};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, StatefulWidget, Widget},
};
use ratatui_core::terminal::Frame;
use web_time::Duration;

const BOOT_LINES: &[&str] = &[
    "INCREMENTAL-CODE BIOS v1.0",
    "(C) 2026",
    "Game by Andreas Tollkötter",
    "Music by Purrplecat",
    "",
    "CPU: NotPython Runtime @ 1.0 GHz  [ OK ]",
    "Memory check: 640K                [ OK ]",
    "Filesystem: ext4                  [ OK ]",
    "",
    "Loading kernel modules ...        [ OK ]",
    "Starting Game v0.1.0 ...",
    "",
    "> System ready.",
];

const POST_TYPING_PAUSE: Duration = Duration::from_millis(1500);

fn total_boot_chars() -> usize {
    BOOT_LINES.iter().map(|l| l.len() + 1).sum()
}

fn fill_black(buf: &mut Buffer, area: Rect) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            buf[(x, y)].set_bg(Color::Black);
            buf[(x, y)].set_fg(Color::Black);
            buf[(x, y)].set_symbol(" ");
        }
    }
}

enum Phase {
    TurningOnOff(CctvAnimationState),
    BootText { elapsed: Duration },
    Done,
}

pub struct RebootScene {
    phase: Phase,
    animation: CctvAnimation,
    char_per_sec: u32,
}

impl RebootScene {
    pub fn new(turning_on: bool, char_per_sec: u32) -> Self {
        Self {
            phase: Phase::TurningOnOff(if turning_on {
                CctvAnimationState::turning_on()
            } else {
                CctvAnimationState::turning_off()
            }),
            animation: CctvAnimation::new(Duration::from_millis(200), Duration::from_millis(200)),
            char_per_sec,
        }
    }

    fn render_boot_text(&self, area: Rect, buf: &mut Buffer, elapsed: Duration) {
        fill_black(buf, area);

        let green = Color::Rgb(0, 200, 0);
        let green_style = Style::default().fg(green);
        let chars_shown = (elapsed * self.char_per_sec).as_secs() as usize;
        let is_all_typed = chars_shown >= total_boot_chars();

        let mut lines = {
            let mut remaining = chars_shown;
            let mut lines: Vec<Line> = Vec::new();
            for &full_line in BOOT_LINES {
                if remaining == 0 {
                    break;
                }
                let slice = full_line.chars().take(remaining).collect::<String>();
                let span = Span::styled(slice, green_style);
                lines.push(Line::from(span));
                remaining = remaining.saturating_sub(full_line.len() + 1);
            }
            lines
        };

        let cursor_visible = (elapsed.as_secs_f32() * 2.0) as u32 % 2 == 0;
        if !is_all_typed && cursor_visible {
            if let Some(last) = lines.last_mut() {
                last.spans.push(Span::styled("▋", green_style));
            }
        }

        Paragraph::new(lines).render(area, buf);
    }
}

impl Scene for RebootScene {
    fn frame(&mut self, _events: &[Event], frame: &mut Frame, time_delta: Duration) -> SceneSwitch {
        let area = frame.area();
        let buf = frame.buffer_mut();

        match &mut self.phase {
            Phase::TurningOnOff(state) => state.update(time_delta),
            Phase::BootText { elapsed } => *elapsed = elapsed.saturating_add(time_delta),
            Phase::Done => {}
        }

        match &mut self.phase {
            Phase::TurningOnOff(state) => {
                self.animation.render(area, buf, state);
            }
            Phase::BootText { elapsed } => {
                let elapsed = *elapsed;
                self.render_boot_text(area, buf, elapsed);
            }
            Phase::Done => {
                fill_black(buf, area);
            }
        }

        let next = match &self.phase {
            Phase::TurningOnOff(state) if self.animation.is_done(state) => Some(Phase::BootText {
                elapsed: Duration::ZERO,
            }),
            Phase::BootText { elapsed } => {
                let total_typing_duration =
                    Duration::from_secs(1) * total_boot_chars() as u32 / self.char_per_sec as u32;
                if elapsed >= &(total_typing_duration + POST_TYPING_PAUSE) {
                    Some(Phase::Done)
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(p) = next {
            self.phase = p;
        }

        match &self.phase {
            Phase::Done => SceneSwitch::SwitchTo(Box::new(HomeTerminalScene::new())),
            _ => SceneSwitch::NoSwitch,
        }
    }
}

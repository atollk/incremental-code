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
    "(C) 2024 NotPython Systems Inc.",
    "",
    "CPU: NotPython Runtime @ 1.0 GHz  [ OK ]",
    "Memory check: 640K                [ OK ]",
    "Filesystem: ext4                  [ OK ]",
    "",
    "Loading kernel modules ...        [ OK ]",
    "Starting NotPython v0.1.0 ...",
    "",
    "> System ready.",
];

const CHARS_PER_SEC: f32 = 40.0;
const POST_TYPING_PAUSE_MS: f32 = 1500.0;

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
    TurningOff { anim: CctvAnimationState },
    BootText { elapsed: Duration },
    Done,
}

pub struct RebootScene {
    phase: Phase,
}

impl RebootScene {
    pub fn new() -> Self {
        Self {
            phase: Phase::TurningOff {
                anim: CctvAnimationState::turning_off(),
            },
        }
    }
}

impl Default for RebootScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for RebootScene {
    fn frame(&mut self, _events: &[Event], frame: &mut Frame, time_delta: Duration) -> SceneSwitch {
        let area = frame.area();
        let buf = frame.buffer_mut();

        match &mut self.phase {
            Phase::TurningOff { anim } => anim.update(time_delta),
            Phase::BootText { elapsed } => *elapsed = elapsed.saturating_add(time_delta),
            Phase::Done => {}
        }

        match &mut self.phase {
            Phase::TurningOff { anim } => {
                CctvAnimation::default().render(area, buf, anim);
            }
            Phase::BootText { elapsed } => {
                render_boot_text(area, buf, *elapsed);
            }
            Phase::Done => {
                fill_black(buf, area);
            }
        }

        let next = match &self.phase {
            Phase::TurningOff { anim } if CctvAnimation::default().is_done(anim) => {
                Some(Phase::BootText {
                    elapsed: Duration::ZERO,
                })
            }
            Phase::BootText { elapsed } => {
                let typing_ms = (total_boot_chars() as f32 / CHARS_PER_SEC) * 1000.0;
                if elapsed.as_secs_f32() * 1000.0 >= typing_ms + POST_TYPING_PAUSE_MS {
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

fn render_boot_text(area: Rect, buf: &mut Buffer, elapsed: Duration) {
    fill_black(buf, area);

    let green = Color::Rgb(0, 200, 0);
    let green_style = Style::default().fg(green);
    let chars_shown = (elapsed.as_secs_f32() * CHARS_PER_SEC) as usize;
    let all_typed = chars_shown >= total_boot_chars();

    let mut remaining = chars_shown;
    let mut lines: Vec<Line> = Vec::new();

    for &text in BOOT_LINES {
        if remaining == 0 {
            break;
        }
        let n = remaining.min(text.len());
        lines.push(Line::from(Span::styled(&text[..n], green_style)));
        remaining = remaining.saturating_sub(text.len() + 1);
    }

    let cursor_visible = (elapsed.as_secs_f32() * 2.0) as u32 % 2 == 0;
    if !all_typed && cursor_visible {
        if let Some(last) = lines.last_mut() {
            last.spans.push(Span::styled("▋", green_style));
        }
    }

    Paragraph::new(lines).render(area, buf);
}

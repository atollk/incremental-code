use std::time::Duration;

use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::StatefulWidget};

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(2)
}

fn fill_rect(buf: &mut Buffer, area: Rect, color: Color) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            buf[(x, y)].set_bg(color);
            buf[(x, y)].set_fg(color);
            buf[(x, y)].set_symbol(" ");
        }
    }
}

fn render_vertical_band(buf: &mut Buffer, area: Rect, active_rows: u16) {
    let h = area.height;
    let w = area.width;
    let top_offset = (h - active_rows) / 2;
    let bottom_offset = h - top_offset - active_rows;
    fill_rect(
        buf,
        Rect::new(area.left(), area.top(), w, top_offset),
        Color::Black,
    );
    fill_rect(
        buf,
        Rect::new(area.left(), area.top() + top_offset, w, active_rows),
        Color::White,
    );
    fill_rect(
        buf,
        Rect::new(
            area.left(),
            area.top() + top_offset + active_rows,
            w,
            bottom_offset,
        ),
        Color::Black,
    );
}

fn render_horizontal_line(buf: &mut Buffer, area: Rect, active_cols: u16) {
    let center_row = area.top() + area.height / 2;
    let left_offset = (area.width - active_cols) / 2;
    fill_rect(buf, area, Color::Black);
    let x_start = area.left() + left_offset;
    for x in x_start..(x_start + active_cols) {
        buf[(x, center_row)].set_bg(Color::White);
        buf[(x, center_row)].set_fg(Color::White);
        buf[(x, center_row)].set_symbol(" ");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Off,
    On,
}

#[derive(Debug, Clone)]
pub struct CctvAnimationState {
    direction: Direction,
    elapsed: Duration,
}

impl CctvAnimationState {
    pub fn turning_off() -> Self {
        Self {
            direction: Direction::Off,
            elapsed: Duration::ZERO,
        }
    }

    pub fn turning_on() -> Self {
        Self {
            direction: Direction::On,
            elapsed: Duration::ZERO,
        }
    }

    pub fn update(&mut self, delta: Duration) {
        self.elapsed = self.elapsed.saturating_add(delta);
    }

    fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

#[derive(Debug, Clone)]
pub struct CctvAnimation {
    pub phase1_duration: Duration,
    pub phase2_duration: Duration,
}

impl CctvAnimation {
    pub fn is_done(&self, state: &CctvAnimationState) -> bool {
        state.elapsed() >= self.phase1_duration + self.phase2_duration
    }

    pub(crate) fn new(phase1_duration: Duration, phase2_duration: Duration) -> Self {
        Self {
            phase1_duration,
            phase2_duration,
        }
    }
}

impl StatefulWidget for &CctvAnimation {
    type State = CctvAnimationState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let elapsed = state.elapsed();
        let w = area.width;
        let h = area.height;

        match state.direction {
            Direction::Off => {
                if elapsed < self.phase1_duration {
                    // Vertical collapse toward a horizontal line
                    let p1 = elapsed.div_duration_f32(self.phase1_duration);
                    let active_rows = ((h as f32 * ease_out(1.0 - p1)).round() as u16)
                        .max(1)
                        .min(h);
                    render_vertical_band(buf, area, active_rows);
                } else {
                    // Horizontal collapse toward a point
                    let p2 = (elapsed - self.phase1_duration)
                        .div_duration_f32(self.phase2_duration)
                        .min(1.0);
                    let active_cols = ((w as f32 * (1.0 - p2)).round() as u16).min(w);
                    render_horizontal_line(buf, area, active_cols);
                }
            }
            Direction::On => {
                if elapsed < self.phase2_duration {
                    // Horizontal expand from a point
                    let p2 = elapsed.div_duration_f32(self.phase2_duration);
                    let active_cols = ((w as f32 * p2).round() as u16).min(w);
                    render_horizontal_line(buf, area, active_cols);
                } else {
                    // Vertical expand from a horizontal line
                    let p1 = (elapsed - self.phase2_duration)
                        .div_duration_f32(self.phase1_duration)
                        .min(1.0);
                    let active_rows = ((h as f32 * ease_out(p1)).round() as u16).max(1).min(h);
                    render_vertical_band(buf, area, active_rows);
                }
            }
        }
    }
}

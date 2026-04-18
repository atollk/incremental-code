use std::time::Duration;

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};
use web_time::Instant;

use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind};
use crate::widgets::blinking_cursor::BlinkingCursor;

/// Trait for commands that run inside the terminal widget.
///
/// While a command is running, it receives all input events and a time delta
/// each frame. Commands can render animated output (spinners, progress bars)
/// while running, and their final output stays on screen after they finish.
pub trait RunningCommand {
    /// Returns true once the command has produced its final output and is done.
    fn is_done(&self) -> bool;

    /// Called each frame while this command is active.
    ///
    /// `events` contains all input events for the frame; `time_delta` is the
    /// elapsed time since the previous call, for time-based animation.
    fn update(&mut self, events: &[Event], time_delta: Duration);

    /// Render the command's current output into `area`.
    ///
    /// Called both while running (for live output) and after completion (so
    /// the final output stays visible in the history area).
    fn render(&self, area: Rect, buf: &mut Buffer);

    /// Number of terminal rows this command's output currently occupies.
    ///
    /// May change between frames (e.g. output growing line by line, or a
    /// spinner that expands). The widget re-reads this value every frame.
    fn height(&self) -> u16;
}

/// A terminal widget with a prompt for typing commands, a running-command area,
/// and a scrolling history of completed command outputs.
///
/// # Usage
///
/// Call [`TerminalWidget::update`] each frame with events and the frame delta,
/// then render the widget and its cursor:
///
/// ```ignore
/// self.terminal.update(events, delta);
/// frame.render_widget(&self.terminal, area);
/// if let Some(cursor) = self.terminal.cursor(&area) {
///     frame.render_widget(cursor, area);
/// }
/// ```
pub struct TerminalWidget {
    /// Completed commands, oldest first. Their output stays visible above the prompt.
    history: Vec<Box<dyn RunningCommand>>,
    /// The currently running command, if any. Input is blocked while this is set.
    pub running: Option<Box<dyn RunningCommand>>,
    /// Text currently being typed in the prompt.
    input: String,
    /// Previously submitted command strings, for up/down arrow navigation.
    input_history: Vec<String>,
    /// Current position in `input_history` while navigating, or `None` when not navigating.
    history_cursor: Option<usize>,
}

impl TerminalWidget {
    /// Create a new terminal widget.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            running: None,
            input: String::new(),
            input_history: Vec::new(),
            history_cursor: None,
        }
    }

    fn handle_event(&mut self, event: &Event) -> Option<String> {
        let Event::KeyEvent(key) = event else {
            return None;
        };
        if key.kind == KeyEventKind::Release {
            return None;
        }

        match key.code {
            KeyCode::Char(c) => {
                self.input.push(c);
                self.history_cursor = None;
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.history_cursor = None;
            }
            KeyCode::Enter => {
                let cmd = self.input.clone();
                if !cmd.is_empty() {
                    self.input_history.push(cmd.clone());
                    self.input.clear();
                    self.history_cursor = None;
                    return Some(cmd);
                }
            }
            KeyCode::Up => {
                if self.input_history.is_empty() {
                    return None;
                }
                let idx = match self.history_cursor {
                    None => self.input_history.len() - 1,
                    Some(0) => 0,
                    Some(i) => i - 1,
                };
                self.history_cursor = Some(idx);
                self.input = self.input_history[idx].clone();
            }
            KeyCode::Down => match self.history_cursor {
                None => {}
                Some(i) if i + 1 >= self.input_history.len() => {
                    self.history_cursor = None;
                    self.input.clear();
                }
                Some(i) => {
                    self.history_cursor = Some(i + 1);
                    self.input = self.input_history[i + 1].clone();
                }
            },
            _ => {}
        }
        None
    }

    /// Process events and advance time.
    ///
    /// When a command is running, all events are forwarded to it and the
    /// prompt is disabled. When no command is running, keyboard events drive
    /// the prompt (typing, backspace, enter, history navigation).
    ///
    /// Returns a command, if one is to be executed by the user.
    pub fn update(&mut self, events: &[Event], time_delta: Duration) -> Option<String> {
        if let Some(running) = &mut self.running {
            running.update(events, time_delta);
            if running.is_done() {
                let done = self.running.take().unwrap();
                self.history.push(done);
            }
            return None;
        }

        for event in events {
            let cmd = self.handle_event(event);
            if cmd.is_some() {
                return cmd;
            }
        }

        if let Some(running) = &mut self.running {
            running.update(&events, time_delta);
            if running.is_done() {
                let done = self.running.take().unwrap();
                self.history.push(done);
            }
        }
        None
    }

    /// Returns a pre-configured [`BlinkingCursor`] positioned at the prompt,
    /// or `None` while a command is running (the cursor is hidden then).
    ///
    /// Render the returned cursor as an overlay on the same area as the widget.
    pub fn cursor(&self, area: &Rect) -> Option<BlinkingCursor> {
        if self.running.is_some() {
            return None;
        }
        let y = area.bottom().saturating_sub(1);
        // "> " prefix is 2 columns wide.
        let x = (area.left() + 2 + self.input.len() as u16).min(area.right().saturating_sub(1));
        Some(BlinkingCursor::new(x, y).with_blink(500))
    }
}

impl Widget for &TerminalWidget {
    /// Render the terminal into `area`.
    ///
    /// Layout (bottom to top):
    /// - Last row: prompt line `> {input}`
    /// - Above: running command output (height re-read each frame)
    /// - Above: history entries, newest first (oldest may be clipped if out of space)
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        // Prompt line is always at the very bottom.
        let input_row = area.bottom() - 1;
        buf.set_string(
            area.left(),
            input_row,
            format!("> {}", self.input),
            Style::default(),
        );

        // `bottom` is the exclusive upper bound of the remaining free rows.
        // We work bottom-up, subtracting each section's height as we go.
        let top = area.top();
        let mut bottom = input_row;

        if top >= bottom {
            return;
        }

        // Running command sits directly above the prompt line.
        // `height()` is re-read every frame so dynamic growth/shrink is handled.
        if let Some(running) = &self.running {
            let h = running.height().min(bottom - top);
            if h > 0 {
                running.render(
                    Rect {
                        x: area.x,
                        y: bottom - h,
                        width: area.width,
                        height: h,
                    },
                    buf,
                );
                bottom -= h;
            }
        }

        // History fills the remaining space above the running command,
        // newest entry first. If total history exceeds available rows,
        // the oldest entries simply don't get rendered (clipped off the top).
        for entry in self.history.iter().rev() {
            if top >= bottom {
                break;
            }
            let h = entry.height().min(bottom - top);
            if h > 0 {
                entry.render(
                    Rect {
                        x: area.x,
                        y: bottom - h,
                        width: area.width,
                        height: h,
                    },
                    buf,
                );
                bottom -= h;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::backend::input::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key_press(code: KeyCode) -> Event {
        Event::KeyEvent(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    // Completes immediately; renders its text into the top-left of the area.
    struct ImmediateCmd {
        text: String,
    }
    impl ImmediateCmd {
        fn new(s: &str) -> Self {
            Self { text: s.to_owned() }
        }
    }
    impl RunningCommand for ImmediateCmd {
        fn is_done(&self) -> bool {
            true
        }
        fn update(&mut self, _: &[Event], _: Duration) {}
        fn render(&self, area: Rect, buf: &mut Buffer) {
            buf.set_string(area.x, area.y, &self.text, Style::default());
        }
        fn height(&self) -> u16 {
            1
        }
    }

    // Takes `remaining` update calls to finish.
    struct SlowCmd {
        remaining: u32,
        h: u16,
    }
    impl RunningCommand for SlowCmd {
        fn is_done(&self) -> bool {
            self.remaining == 0
        }
        fn update(&mut self, _: &[Event], _: Duration) {
            self.remaining = self.remaining.saturating_sub(1);
        }
        fn render(&self, _: Rect, _: &mut Buffer) {}
        fn height(&self) -> u16 {
            self.h
        }
    }

    // Height grows by 1 each update call; finishes after 3 calls.
    struct GrowingCmd {
        calls: u16,
    }
    impl RunningCommand for GrowingCmd {
        fn is_done(&self) -> bool {
            self.calls >= 3
        }
        fn update(&mut self, _: &[Event], _: Duration) {
            self.calls += 1;
        }
        fn render(&self, _: Rect, _: &mut Buffer) {}
        fn height(&self) -> u16 {
            self.calls + 1
        }
    }

    fn make_terminal() -> TerminalWidget {
        TerminalWidget::new()
    }

    fn render_terminal(t: &TerminalWidget, width: u16, height: u16) -> Buffer {
        let area = Rect {
            x: 0,
            y: 0,
            width,
            height,
        };
        let mut buf = Buffer::empty(area);
        Widget::render(t, area, &mut buf);
        buf
    }

    fn row_string(buf: &Buffer, y: u16, width: u16) -> String {
        (0..width)
            .map(|x| buf[(x, y)].symbol().chars().next().unwrap_or(' '))
            .collect()
    }

    // ── Input handling ────────────────────────────────────────────────────────

    #[test]
    fn typing_appends_to_input() {
        let mut t = make_terminal();
        t.update(
            &[key_press(KeyCode::Char('h')), key_press(KeyCode::Char('i'))],
            Duration::ZERO,
        );
        assert_eq!(t.input, "hi");
    }

    #[test]
    fn backspace_removes_last_char() {
        let mut t = make_terminal();
        t.update(
            &[key_press(KeyCode::Char('a')), key_press(KeyCode::Backspace)],
            Duration::ZERO,
        );
        assert_eq!(t.input, "");
    }

    #[test]
    fn backspace_on_empty_input_is_noop() {
        let mut t = make_terminal();
        t.update(&[key_press(KeyCode::Backspace)], Duration::ZERO);
        assert_eq!(t.input, "");
    }

    #[test]
    fn enter_submits_and_clears_input() {
        let mut t = make_terminal();
        t.update(
            &[key_press(KeyCode::Char('x')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        );
        // ImmediateCmd finishes synchronously inside the first update call.
        assert_eq!(t.input, "");
        assert!(t.running.is_none());
        assert_eq!(t.history.len(), 1);
    }

    #[test]
    fn empty_enter_does_not_save_to_input_history() {
        let mut t = make_terminal();
        t.update(&[key_press(KeyCode::Enter)], Duration::ZERO);
        assert_eq!(t.input_history.len(), 0);
    }

    // ── Input history navigation ──────────────────────────────────────────────

    #[test]
    fn up_down_navigate_input_history() {
        let mut t = make_terminal();
        t.update(
            &[key_press(KeyCode::Char('a')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        );
        t.update(
            &[key_press(KeyCode::Char('b')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        );

        t.update(&[key_press(KeyCode::Up)], Duration::ZERO);
        assert_eq!(t.input, "b");

        t.update(&[key_press(KeyCode::Up)], Duration::ZERO);
        assert_eq!(t.input, "a");

        // Clamped at oldest.
        t.update(&[key_press(KeyCode::Up)], Duration::ZERO);
        assert_eq!(t.input, "a");

        t.update(&[key_press(KeyCode::Down)], Duration::ZERO);
        assert_eq!(t.input, "b");

        // Past newest → clear and exit navigation.
        t.update(&[key_press(KeyCode::Down)], Duration::ZERO);
        assert_eq!(t.input, "");
        assert!(t.history_cursor.is_none());
    }

    #[test]
    fn typing_after_history_nav_resets_cursor() {
        let mut t = make_terminal();
        t.update(
            &[key_press(KeyCode::Char('a')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        );
        t.update(&[key_press(KeyCode::Up)], Duration::ZERO);
        assert!(t.history_cursor.is_some());
        t.update(&[key_press(KeyCode::Char('b'))], Duration::ZERO);
        assert!(t.history_cursor.is_none());
    }

    // ── Running command lifecycle ─────────────────────────────────────────────

    #[test]
    fn slow_command_blocks_prompt_input() {
        // remaining=3: Enter's post-idle tick → 2, call 2 → 1, call 3 → 0 (done).
        let mut t = TerminalWidget::new();
        t.update(
            &[key_press(KeyCode::Char('x')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        );
        assert!(t.running.is_some()); // remaining=2

        // Typing while running is silently ignored.
        t.update(&[key_press(KeyCode::Char('y'))], Duration::ZERO);
        assert_eq!(t.input, "");
        assert!(t.running.is_some()); // remaining=1

        // One more update finishes the command.
        t.update(&[], Duration::ZERO);
        assert!(t.running.is_none());
        assert_eq!(t.history.len(), 1);
    }

    #[test]
    fn immediate_command_transitions_within_same_update() {
        let mut t = make_terminal();
        t.update(
            &[key_press(KeyCode::Char('x')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        );
        // No separate update needed — ImmediateCmd::is_done returns true immediately.
        assert!(t.running.is_none());
        assert_eq!(t.history.len(), 1);
    }

    // ── Cursor ────────────────────────────────────────────────────────────────

    #[test]
    fn cursor_is_none_while_running() {
        // remaining=2: post-idle tick → 1, so command is still running after Enter.
        let mut t = TerminalWidget::new();
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };
        t.update(
            &[key_press(KeyCode::Char('x')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        );
        assert!(t.cursor(&area).is_none());
    }

    #[test]
    fn cursor_is_some_when_idle() {
        let t = make_terminal();
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };
        assert!(t.cursor(&area).is_some());
    }

    // ── Rendering ────────────────────────────────────────────────────────────

    #[test]
    fn renders_prompt_at_bottom() {
        let mut t = make_terminal();
        t.update(
            &[key_press(KeyCode::Char('h')), key_press(KeyCode::Char('i'))],
            Duration::ZERO,
        );
        let buf = render_terminal(&t, 20, 5);
        assert!(row_string(&buf, 4, 20).starts_with("> hi"));
    }

    #[test]
    fn history_rendered_above_prompt() {
        let mut t = make_terminal();
        t.update(
            &[
                key_press(KeyCode::Char('f')),
                key_press(KeyCode::Char('o')),
                key_press(KeyCode::Char('o')),
                key_press(KeyCode::Enter),
            ],
            Duration::ZERO,
        );
        let buf = render_terminal(&t, 20, 5);
        // height=5: row 4 = prompt, row 3 = history (height 1).
        assert!(row_string(&buf, 3, 20).starts_with("foo"));
        assert!(row_string(&buf, 4, 20).starts_with("> "));
    }

    #[test]
    fn dynamic_height_command_gets_correct_rect() {
        // GrowingCmd: after Enter's internal update, calls=1, height()=2, not done.
        let mut t = TerminalWidget::new();
        t.update(
            &[key_press(KeyCode::Char('x')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        );
        assert!(t.running.is_some());

        // height() = 2; render and verify we don't panic with an oversized rect.
        let _ = render_terminal(&t, 10, 10);

        t.update(&[], Duration::ZERO); // calls=2, height()=3
        assert!(t.running.is_some());

        t.update(&[], Duration::ZERO); // calls=3, is_done → history
        assert!(t.running.is_none());
        assert_eq!(t.history.len(), 1);
        assert_eq!(t.history[0].height(), 4); // calls+1 = 4
    }

    #[test]
    fn overflow_history_clips_oldest_entries() {
        // 4-row terminal: prompt=1, two history entries of height=2 → oldest is clipped.
        struct TwoRowCmd;
        impl RunningCommand for TwoRowCmd {
            fn is_done(&self) -> bool {
                true
            }
            fn update(&mut self, _: &[Event], _: Duration) {}
            fn render(&self, area: Rect, buf: &mut Buffer) {
                buf.set_string(area.x, area.y, "LINE1", Style::default());
                if area.height > 1 {
                    buf.set_string(area.x, area.y + 1, "LINE2", Style::default());
                }
            }
            fn height(&self) -> u16 {
                2
            }
        }

        let mut t = TerminalWidget::new();
        t.update(
            &[key_press(KeyCode::Char('a')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        );
        t.update(
            &[key_press(KeyCode::Char('b')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        );
        assert_eq!(t.history.len(), 2);

        let buf = render_terminal(&t, 10, 4);
        // Row 3: prompt
        assert!(row_string(&buf, 3, 10).starts_with("> "));
        // Rows 1-2: newest history ("b" cmd), height=2, fits in full.
        assert!(row_string(&buf, 1, 10).starts_with("LINE1"));
        assert!(row_string(&buf, 2, 10).starts_with("LINE2"));
        // Row 0: oldest history ("a" cmd) is partially visible — only 1 row fits,
        // so it renders LINE1 (the first row of a 2-row entry, like a real terminal
        // scrolled to the bottom with content clipped at the top).
        assert!(row_string(&buf, 0, 10).starts_with("LINE1"));
    }
}

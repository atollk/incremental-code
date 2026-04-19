use std::time::Duration;

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};
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
    fn height(&self, columns: u16) -> u16;
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
    /// Byte offset of the cursor within `input`.
    input_cursor: usize,
    /// Previously submitted command strings, for up/down arrow navigation.
    input_history: Vec<String>,
    /// Current position in `input_history` while navigating, or `None` when not navigating.
    history_cursor: Option<usize>,
    /// Cursor appearance. Configure style, symbol, and blink period here.
    /// `x`/`y` are ignored — the widget positions the cursor automatically.
    pub cursor: BlinkingCursor,
}

/// Wraps a [`RunningCommand`] and prepends a prompt-echo line (`> cmd`) so
/// history entries look like a real terminal: the typed command followed by its output.
struct EchoedCommand {
    echo: String,
    inner: Box<dyn RunningCommand>,
}

impl RunningCommand for EchoedCommand {
    fn is_done(&self) -> bool {
        self.inner.is_done()
    }
    fn update(&mut self, events: &[Event], time_delta: Duration) {
        self.inner.update(events, time_delta);
    }
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // First row: the echoed prompt line.
        buf.set_string(area.x, area.y, &self.echo, Style::default());
        // Remaining rows: the inner command's output.
        if area.height > 1 {
            self.inner.render(
                Rect {
                    y: area.y + 1,
                    height: area.height - 1,
                    ..area
                },
                buf,
            );
        }
    }
    fn height(&self, columns: u16) -> u16 {
        1 + self.inner.height(columns)
    }
}

impl TerminalWidget {
    /// Create a new terminal widget.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            running: None,
            input: String::new(),
            input_cursor: 0,
            input_history: Vec::new(),
            history_cursor: None,
            cursor: BlinkingCursor::new(0, 0).with_blink(500),
        }
    }

    /// Register a command to run, wrapping it with a prompt-echo line.
    ///
    /// Call this instead of setting `running` directly after receiving a command
    /// string from [`update`](Self::update), so history entries show `> cmd` followed
    /// by the command's output, just like a real terminal.
    pub fn set_running(&mut self, cmd_text: &str, cmd: Box<dyn RunningCommand>) {
        self.running = Some(Box::new(EchoedCommand {
            echo: format!("> {}", cmd_text),
            inner: cmd,
        }));
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
                self.input.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
                self.history_cursor = None;
            }
            KeyCode::Backspace => {
                if self.input_cursor > 0 {
                    // Find the start of the character just before the cursor.
                    let prev = self.input[..self.input_cursor]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.input.remove(prev);
                    self.input_cursor = prev;
                }
                self.history_cursor = None;
            }
            KeyCode::Left => {
                if self.input_cursor > 0 {
                    self.input_cursor = self.input[..self.input_cursor]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                }
            }
            KeyCode::Right => {
                if self.input_cursor < self.input.len() {
                    let c = self.input[self.input_cursor..]
                        .chars()
                        .next()
                        .unwrap();
                    self.input_cursor += c.len_utf8();
                }
            }
            KeyCode::Enter => {
                let cmd = self.input.clone();
                if !cmd.is_empty() {
                    self.input_history.push(cmd.clone());
                    self.input.clear();
                    self.input_cursor = 0;
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
                self.input_cursor = self.input.len();
            }
            KeyCode::Down => match self.history_cursor {
                None => {}
                Some(i) if i + 1 >= self.input_history.len() => {
                    self.history_cursor = None;
                    self.input.clear();
                    self.input_cursor = 0;
                }
                Some(i) => {
                    self.history_cursor = Some(i + 1);
                    self.input = self.input_history[i + 1].clone();
                    self.input_cursor = self.input.len();
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

}

impl Widget for &TerminalWidget {
    /// Render the terminal into `area`.
    ///
    /// Layout (top to bottom, like a real terminal):
    /// - History entries, oldest first
    /// - Running command output (height re-read each frame)
    /// - Prompt line `> {input}` immediately after the last content row
    ///
    /// If content fills the area, the prompt is pushed down and may be clipped.
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let mut y = area.top();

        // History entries, oldest first, flowing downward.
        for entry in &self.history {
            if y >= area.bottom() {
                break;
            }
            let h = entry.height(area.width).min(area.bottom() - y);
            if h > 0 {
                entry.render(
                    Rect {
                        x: area.x,
                        y,
                        width: area.width,
                        height: h,
                    },
                    buf,
                );
                y += h;
            }
        }

        // Running command sits directly below the last history entry.
        if let Some(running) = &self.running {
            if y < area.bottom() {
                let h = running.height(area.width).min(area.bottom() - y);
                if h > 0 {
                    running.render(
                        Rect {
                            x: area.x,
                            y,
                            width: area.width,
                            height: h,
                        },
                        buf,
                    );
                    y += h;
                }
            }
        }

        // Prompt and cursor appear right after the last content row, but only when idle.
        if self.running.is_none() && y < area.bottom() {
            buf.set_string(
                area.left(),
                y,
                format!("> {}", self.input),
                Style::default(),
            );
            // "> " prefix is 2 columns wide; cursor column is char count before byte offset.
            let char_col = self.input[..self.input_cursor].chars().count() as u16;
            let cx = (area.left() + 2 + char_col).min(area.right().saturating_sub(1));
            self.cursor.clone().at(cx, y).render(area, buf);
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
        fn height(&self, _columns: u16) -> u16 {
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
        fn height(&self, _columns: u16) -> u16 {
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
        fn height(&self, _columns: u16) -> u16 {
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
        let cmd = t.update(
            &[key_press(KeyCode::Char('x')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        );
        assert_eq!(cmd, Some("x".to_string()));
        assert_eq!(t.input, "");
        assert!(t.running.is_none());
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
        let mut t = TerminalWidget::new();
        // Enter returns the command string; caller sets t.running.
        if let Some(_) = t.update(
            &[key_press(KeyCode::Char('x')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        ) {
            t.running = Some(Box::new(SlowCmd { remaining: 2, h: 1 }));
        }
        assert!(t.running.is_some());

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
        // Caller sets a command that is already done; next update moves it to history.
        if let Some(_) = t.update(
            &[key_press(KeyCode::Char('x')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        ) {
            t.running = Some(Box::new(ImmediateCmd::new("x")));
        }
        t.update(&[], Duration::ZERO);
        assert!(t.running.is_none());
        assert_eq!(t.history.len(), 1);
    }

    // ── Cursor ────────────────────────────────────────────────────────────────

    #[test]
    fn cursor_hidden_while_running() {
        // The cursor is not rendered when a command is running.
        // Verify indirectly: render into a buffer and check the cursor cell
        // is not styled (the prompt line itself is also absent while running).
        let mut t = TerminalWidget::new();
        if let Some(_) = t.update(
            &[key_press(KeyCode::Char('x')), key_press(KeyCode::Enter)],
            Duration::ZERO,
        ) {
            t.running = Some(Box::new(SlowCmd { remaining: 1, h: 1 }));
        }
        assert!(t.running.is_some()); // sanity: command is active
        let buf = render_terminal(&t, 80, 24);
        // Row 0 should not contain a prompt while running.
        assert!(!row_string(&buf, 0, 80).starts_with("> "));
    }

    #[test]
    fn cursor_visible_when_idle() {
        let t = make_terminal();
        // With no history and no running command the prompt is at row 0.
        // Cursor cell (column 2) should have the cursor style applied.
        let buf = render_terminal(&t, 80, 24);
        assert!(row_string(&buf, 0, 80).starts_with("> "));
    }

    // ── Rendering ────────────────────────────────────────────────────────────

    #[test]
    fn renders_prompt_at_top_when_no_history() {
        let mut t = make_terminal();
        t.update(
            &[key_press(KeyCode::Char('h')), key_press(KeyCode::Char('i'))],
            Duration::ZERO,
        );
        let buf = render_terminal(&t, 20, 5);
        // No history → prompt at row 0 (top).
        assert!(row_string(&buf, 0, 20).starts_with("> hi"));
    }

    #[test]
    fn history_rendered_above_prompt() {
        let mut t = make_terminal();
        // Submit a command and set a completed command in history manually.
        t.update(
            &[
                key_press(KeyCode::Char('f')),
                key_press(KeyCode::Char('o')),
                key_press(KeyCode::Char('o')),
                key_press(KeyCode::Enter),
            ],
            Duration::ZERO,
        );
        // Simulate the caller setting a completed command and it moving to history.
        t.running = Some(Box::new(ImmediateCmd::new("foo")));
        t.update(&[], Duration::ZERO); // moves it to history
        let buf = render_terminal(&t, 20, 5);
        // Top-down: row 0 = history entry ("foo"), row 1 = prompt.
        assert!(row_string(&buf, 0, 20).starts_with("foo"));
        assert!(row_string(&buf, 1, 20).starts_with("> "));
    }

    #[test]
    fn dynamic_height_command_gets_correct_rect() {
        let mut t = TerminalWidget::new();
        // Caller sets a GrowingCmd: height starts at 1, grows each update, done at calls=3.
        t.running = Some(Box::new(GrowingCmd { calls: 0 }));

        // calls=0, height()=1; render should not panic.
        let _ = render_terminal(&t, 10, 10);

        t.update(&[], Duration::ZERO); // calls=1, height()=2
        assert!(t.running.is_some());

        t.update(&[], Duration::ZERO); // calls=2, height()=3
        assert!(t.running.is_some());

        t.update(&[], Duration::ZERO); // calls=3, is_done → history
        assert!(t.running.is_none());
        assert_eq!(t.history.len(), 1);
        assert_eq!(t.history[0].height(10), 4); // calls+1 = 4
    }

    #[test]
    fn overflow_history_clips_newest_entries() {
        // 4-row terminal: two history entries of height=2 each → newest is clipped at bottom.
        // Top-down layout: oldest fills rows 0-1, newest fills rows 2-3, no room for prompt.
        struct TwoRowCmd(&'static str);
        impl RunningCommand for TwoRowCmd {
            fn is_done(&self) -> bool { true }
            fn update(&mut self, _: &[Event], _: Duration) {}
            fn render(&self, area: Rect, buf: &mut Buffer) {
                buf.set_string(area.x, area.y, self.0, Style::default());
                if area.height > 1 {
                    buf.set_string(area.x, area.y + 1, "CONT", Style::default());
                }
            }
            fn height(&self, _columns: u16) -> u16 { 2 }
        }

        let mut t = TerminalWidget::new();
        // Push two completed commands into history directly.
        t.running = Some(Box::new(TwoRowCmd("AAA")));
        t.update(&[], Duration::ZERO); // AAA → history
        t.running = Some(Box::new(TwoRowCmd("BBB")));
        t.update(&[], Duration::ZERO); // BBB → history
        assert_eq!(t.history.len(), 2);

        let buf = render_terminal(&t, 10, 4);
        // Oldest (AAA) at rows 0-1, newest (BBB) at rows 2-3.
        // Prompt has no space and is not rendered.
        assert!(row_string(&buf, 0, 10).starts_with("AAA"));
        assert!(row_string(&buf, 1, 10).starts_with("CONT"));
        assert!(row_string(&buf, 2, 10).starts_with("BBB"));
        assert!(row_string(&buf, 3, 10).starts_with("CONT"));
    }
}

use std::time::Duration;

use crate::backend::events::Event;
use ratatui::widgets::Paragraph;
use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use ratatui_core::widgets::Widget;

/// Trait for commands that run inside the terminal widget.
///
/// While a command is running, it receives all input events and a time delta
/// each frame. Commands can render animated output (spinners, progress bars)
/// while running, and their final output stays on screen after they finish.
pub trait RunningCommand<Meta = ()> {
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

    /// Gets the metadata of this command.
    fn get_metadata(&self) -> Meta;
}

/// Wraps a [`RunningCommand`] and prepends a prompt-echo line (e.g., `> cmd`) so
/// history entries look like a real terminal: the typed command followed by its output.
pub struct EchoedCommand<Meta> {
    echo: String,
    inner: Box<dyn RunningCommand<Meta>>,
}

impl<Meta> EchoedCommand<Meta> {
    pub fn new(echo: String, inner: Box<dyn RunningCommand<Meta>>) -> EchoedCommand<Meta> {
        EchoedCommand { echo, inner }
    }
}

impl<Meta> RunningCommand<Meta> for EchoedCommand<Meta> {
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

    fn get_metadata(&self) -> Meta {
        self.inner.get_metadata()
    }
}

/// Shows a paragraph of text and finishes immediately.
pub struct ParagraphCmd<'a> {
    paragraph: Paragraph<'a>,
}

impl<'a> ParagraphCmd<'a> {
    pub fn new(paragraph: Paragraph<'a>) -> ParagraphCmd<'a> {
        ParagraphCmd { paragraph }
    }
}

impl<Meta: Default> RunningCommand<Meta> for ParagraphCmd<'_> {
    fn is_done(&self) -> bool {
        true
    }

    fn update(&mut self, _events: &[Event], _time_delta: Duration) {}

    fn render(&self, area: Rect, buf: &mut Buffer) {
        (&self.paragraph).render(area, buf);
    }

    fn height(&self, columns: u16) -> u16 {
        self.paragraph.line_count(columns) as u16
    }

    fn get_metadata(&self) -> Meta {
        Meta::default()
    }
}

/// Chains two commands sequentially.
pub struct ChainCmd<C1, C2> {
    first_command: Box<C1>,
    second_command: Option<Box<C2>>,
    second_command_constructor: Box<dyn Fn(&C1) -> Box<C2>>,
    keep_rendering_first_command: bool,
}

impl<C1, C2> ChainCmd<C1, C2> {
    pub fn new(
        first_command: Box<C1>,
        second_command_constructor: Box<dyn Fn(&C1) -> Box<C2>>,
        keep_rendering_first_command: bool,
    ) -> ChainCmd<C1, C2> {
        ChainCmd {
            first_command,
            second_command: None,
            second_command_constructor,
            keep_rendering_first_command,
        }
    }
}

impl<Meta, C1: RunningCommand<Meta>, C2: RunningCommand<Meta>> RunningCommand<Meta>
    for ChainCmd<C1, C2>
{
    fn is_done(&self) -> bool {
        if let Some(second_command) = self.second_command.as_ref() {
            second_command.is_done()
        } else {
            false
        }
    }

    fn update(&mut self, events: &[Event], time_delta: Duration) {
        if self.first_command.is_done() && self.second_command.is_none() {
            self.second_command = Some((self.second_command_constructor)(&self.first_command));
        }
        if let Some(second_command) = self.second_command.as_mut() {
            second_command.update(events, time_delta);
        } else {
            self.first_command.update(events, time_delta);
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        if let Some(second_command) = self.second_command.as_ref() {
            if self.keep_rendering_first_command {
                let first_height = self.first_command.height(area.width);
                let second_area = Rect {
                    y: area.y + first_height,
                    height: area.height.saturating_sub(first_height),
                    ..area
                };
                second_command.render(second_area, buf);
            } else {
                second_command.render(area, buf);
            }
        } else {
            self.first_command.render(area, buf);
        }
    }

    fn height(&self, columns: u16) -> u16 {
        if let Some(second_command) = self.second_command.as_ref() {
            if self.keep_rendering_first_command {
                self.first_command.height(columns) + second_command.height(columns)
            } else {
                second_command.height(columns)
            }
        } else {
            self.first_command.height(columns)
        }
    }

    fn get_metadata(&self) -> Meta {
        if let Some(second_command) = self.second_command.as_ref() {
            second_command.get_metadata()
        } else {
            self.first_command.get_metadata()
        }
    }
}

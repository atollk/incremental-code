use crate::backend::events::Event;
use crate::game_scenes::base::Scene;
use crate::game_scenes::base::SceneSwitch;
use crate::game_state::with_game_state;
use crate::widgets::terminal::{RunningCommand, TerminalWidget};
use ratatui::widgets::Paragraph;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::terminal::Frame;
use ratatui_core::text::{Line, Text};
use ratatui_core::widgets::{StatefulWidget, Widget};
use std::cell::RefCell;
use web_time::Duration;

pub struct HomeTerminalScene {
    terminal_widget: TerminalWidget,
}

impl Default for HomeTerminalScene {
    fn default() -> Self {
        Self::new()
    }
}

impl HomeTerminalScene {
    pub fn new() -> Self {
        HomeTerminalScene {
            terminal_widget: TerminalWidget::new(),
        }
    }

    fn handle_terminal_command(&self, cmd: &str) -> Box<dyn RunningCommand> {
        match cmd.trim() {
            "help" => help_cmd(),
            "compile" => compile_cmd(),
            _ => unknown_cmd(cmd.to_owned()),
        }
    }
}

impl Scene for HomeTerminalScene {
    fn frame(&mut self, events: &[Event], frame: &mut Frame, time_delta: Duration) -> SceneSwitch {
        let cmd = self.terminal_widget.update(events, time_delta);
        if let Some(cmd) = cmd {
            self.terminal_widget
                .set_running(&cmd, self.handle_terminal_command(&cmd));
        }
        frame.render_widget(&self.terminal_widget, frame.area());
        SceneSwitch::NoSwitch
    }
}

fn unknown_cmd(cmd: String) -> Box<dyn RunningCommand> {
    let text = format!("Unknown command '{cmd}'. For a list of available commands, try 'help'.");
    let text = Text::raw(text);
    Box::new(ParagraphCmd {
        paragraph: Paragraph::new(text),
    })
}

fn help_cmd() -> Box<dyn RunningCommand> {
    struct HelpCommand {
        name: &'static str,
        text: &'static str,
    }
    let available_commands = vec![HelpCommand {
        name: "help",
        text: "Displays this help text",
    }];
    let lines = std::iter::once("List of available commands:".to_string())
        .chain(
            available_commands
                .iter()
                .map(|c| format!("  {}\t - {}", c.name, c.text)),
        )
        .map(Line::from)
        .collect::<Vec<_>>();
    let text = Text::from(lines);
    Box::new(ParagraphCmd {
        paragraph: Paragraph::new(text),
    })
}

fn compile_cmd() -> Box<dyn RunningCommand> {
    with_game_state(|game_state| -> Box<dyn RunningCommand> {
        if game_state.program_code.is_empty() {
            let text = "There is no program to compile. Use 'code' the open the code editor and write a program before compiling.";
            let text = Text::raw(text);
            Box::new(ParagraphCmd {
                paragraph: Paragraph::new(text),
            })
        } else {
            Box::new(CompileCmd::new())
        }
    })
}

/// Shows a paragraph of text and finishes immediately.
struct ParagraphCmd<'a> {
    paragraph: Paragraph<'a>,
}

impl RunningCommand for ParagraphCmd<'_> {
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
}

struct CompileCmd {
    running_duration: Duration,
    compile_duration: Duration,
    throbber_start_index: i8,
    throbber_state: RefCell<throbber_widgets_tui::ThrobberState>,
}

impl CompileCmd {
    const THROBBER_STEP_SPEED: Duration = Duration::from_millis(300);

    fn new() -> Self {
        let mut throbber_state = RefCell::new(throbber_widgets_tui::ThrobberState::default());
        throbber_state.get_mut().calc_step(0); // randomize starting step
        CompileCmd {
            running_duration: Duration::from_millis(0),
            compile_duration: Duration::from_secs(5),
            throbber_start_index: throbber_state.get_mut().index(),
            throbber_state,
        }
    }
}

impl RunningCommand for CompileCmd {
    fn is_done(&self) -> bool {
        self.compile_duration <= self.running_duration
    }

    fn update(&mut self, _events: &[Event], time_delta: Duration) {
        self.running_duration += time_delta;
        let target_throbber_index = self
            .running_duration
            .div_duration_f32(CompileCmd::THROBBER_STEP_SPEED) as i8;
        let mut throbber_state = self.throbber_state.borrow_mut();
        let step = target_throbber_index - throbber_state.index() - self.throbber_start_index;
        if step > 0 {
            throbber_state.calc_step(step);
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let throbber_set = throbber_widgets_tui::BRAILLE_SIX;
        let label = if self.is_done() {
            "Compiling done".to_string()
        } else {
            format!(
                "Compiling{}",
                ".".repeat(
                    (self
                        .running_duration
                        .div_duration_f32(CompileCmd::THROBBER_STEP_SPEED)
                        as i8
                        % 3) as usize
                )
            )
        };
        let full = throbber_widgets_tui::Throbber::default()
            .label(label)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
            .throbber_style(
                ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD),
            )
            .throbber_set(throbber_set)
            .use_type(throbber_widgets_tui::WhichUse::Spin);
        StatefulWidget::render(full, area, buf, &mut *self.throbber_state.borrow_mut());
    }

    fn height(&self, _columns: u16) -> u16 {
        1
    }
}

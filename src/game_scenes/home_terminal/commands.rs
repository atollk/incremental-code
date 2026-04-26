use crate::backend::events::Event;
use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::code_editor::CodeEditorScene;
use crate::game_state::with_game_state;
use crate::widgets::terminal::{ChainCmd, ParagraphCmd, RunningCommand};
use anyhow::anyhow;
use language::{compile, parse_program};
use ratatui::widgets::Paragraph;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::text::{Line, Text};
use ratatui_core::widgets::StatefulWidget;
use std::cell::RefCell;
use std::time::Duration;

pub struct Command {
    pub(crate) name: &'static str,
    help_description: &'static str,
    pub(crate) runner: fn() -> Box<dyn RunningCommand<SceneSwitch>>,
}

pub fn command_list() -> Vec<Command> {
    vec![
        Command {
            name: "help",
            help_description: "Displays this help text",
            runner: help_cmd,
        },
        Command {
            name: "exit",
            help_description: "Exits the game",
            runner: exit_cmd,
        },
        Command {
            name: "code",
            help_description: "Opens the code editor to write or modify your program",
            runner: code_cmd,
        },
        Command {
            name: "compile",
            help_description: "Compiles the program code to make it executable",
            runner: compile_cmd,
        },
    ]
}

pub fn unknown_cmd(cmd: String) -> Box<dyn RunningCommand<SceneSwitch>> {
    let text = format!("Unknown command '{cmd}'. For a list of available commands, try 'help'.");
    let text = Text::raw(text);
    Box::new(ParagraphCmd::new(Paragraph::new(text)))
}

fn help_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    let available_commands = command_list();
    let lines = std::iter::once("List of available commands:".to_string())
        .chain(
            available_commands
                .iter()
                .map(|c| format!("  {}\t - {}", c.name, c.help_description)),
        )
        .map(Line::from)
        .collect::<Vec<_>>();
    let text = Text::from(lines);
    Box::new(ParagraphCmd::new(Paragraph::new(text)))
}

fn exit_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    Box::new(ExitCmd {})
}

struct ExitCmd {}

impl RunningCommand<SceneSwitch> for ExitCmd {
    fn is_done(&self) -> bool {
        true
    }

    fn update(&mut self, _events: &[Event], _time_delta: Duration) {}

    fn render(&self, _area: Rect, _buf: &mut Buffer) {}

    fn height(&self, _columns: u16) -> u16 {
        0
    }

    fn get_metadata(&self) -> SceneSwitch {
        SceneSwitch::ExitGame
    }
}

fn code_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    Box::new(CodeCmd {})
}

struct CodeCmd {}

impl RunningCommand<SceneSwitch> for CodeCmd {
    fn is_done(&self) -> bool {
        true
    }

    fn update(&mut self, _events: &[Event], _time_delta: Duration) {}

    fn render(&self, _area: Rect, _buf: &mut Buffer) {}

    fn height(&self, _columns: u16) -> u16 {
        0
    }

    fn get_metadata(&self) -> SceneSwitch {
        SceneSwitch::SwitchTo(Box::new(CodeEditorScene::new()))
    }
}

fn compile_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    with_game_state(|game_state| -> Box<dyn RunningCommand<SceneSwitch>> {
        if game_state.program_code.is_empty() {
            let text = "There is no program to compile. Use 'code' to open the code editor and write a program before compiling.";
            let text = Text::raw(text);
            Box::new(ParagraphCmd::new(Paragraph::new(text)))
        } else {
            Box::new(ChainCmd::new(
                Box::new(CompileCmd::new()),
                Box::new(|compile_cmd| {
                    let result = compile_cmd
                        .result
                        .as_ref()
                        .expect("compile command to finish");
                    let paragraph: Paragraph<'static> = if let Err(e) = result {
                        Paragraph::new(e.to_string())
                    } else {
                        let text = with_game_state(|game_state| {
                            format!("Compilation successful., {:?}", game_state.compiled_program)
                        });
                        Paragraph::new(text)
                    };
                    Box::new(ParagraphCmd::new(paragraph))
                }),
                true,
            ))
        }
    })
}

struct CompileCmd {
    // when waiting
    running_duration: Duration,
    compile_duration: Duration,
    throbber_state: RefCell<throbber_widgets_tui::ThrobberState>,
    // after waiting
    result: Option<anyhow::Result<()>>,
}

impl CompileCmd {
    const THROBBER_STEP_SPEED: Duration = Duration::from_millis(300);
    const THROBBER_SET: throbber_widgets_tui::Set = throbber_widgets_tui::BRAILLE_SIX;

    fn new() -> Self {
        let mut throbber_state = RefCell::new(throbber_widgets_tui::ThrobberState::default());
        throbber_state.get_mut().calc_step(0); // randomize animation start
        CompileCmd {
            running_duration: Duration::from_millis(0),
            compile_duration: Duration::from_millis(500),
            throbber_state,
            result: None,
        }
    }
}

impl RunningCommand<SceneSwitch> for CompileCmd {
    fn is_done(&self) -> bool {
        self.result.is_some()
    }

    fn update(&mut self, _events: &[Event], time_delta: Duration) {
        if self.compile_duration <= self.running_duration {
            if self.result.is_none() {
                self.result = with_game_state(|game_state| {
                    // TODO: run this while actually waiting, not just at the end
                    let parsed = parse_program(&game_state.program_code);
                    let (compiled_program, result) = match parsed {
                        Ok(parsed) => (Some(compile(&parsed)), Some(Ok(()))),
                        Err(richs) => {
                            let errs = richs.into_iter().map(|rich| Err(anyhow!("{rich}")));
                            (None, Some(errs.collect()))
                        }
                    };
                    game_state.compiled_program = compiled_program;
                    result
                });
            }
        } else {
            // Animate loading
            let throbber_animation_steps =
                |d: Duration| d.div_duration_f32(CompileCmd::THROBBER_STEP_SPEED) as i8;
            let old_duration = self.running_duration;
            self.running_duration += time_delta;
            let throbber_animation_step_div = throbber_animation_steps(self.running_duration)
                - throbber_animation_steps(old_duration);
            if throbber_animation_step_div > 0 {
                self.throbber_state
                    .borrow_mut()
                    .calc_step(throbber_animation_step_div);
            }
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let label = if self.compile_duration <= self.running_duration {
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
            .throbber_set(CompileCmd::THROBBER_SET)
            .use_type(throbber_widgets_tui::WhichUse::Spin);
        StatefulWidget::render(full, area, buf, &mut *self.throbber_state.borrow_mut());
    }

    fn height(&self, _columns: u16) -> u16 {
        1
    }

    fn get_metadata(&self) -> SceneSwitch {
        SceneSwitch::NoSwitch
    }
}

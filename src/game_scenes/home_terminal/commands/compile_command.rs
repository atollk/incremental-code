use crate::backend::events::Event;
use crate::game_scenes::base::SceneSwitch;
use crate::game_state::{CompiledProgram, with_game_state, with_game_state_mut};
use crate::widgets::terminal::{ChainCmd, ParagraphCmd, RunningCommand};
use anyhow::anyhow;
use language::{PredefinedFunction, ProgramValue, compile_with_meta, parse_program};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::text::Text;
use ratatui_core::widgets::StatefulWidget;
use ratatui_widgets::paragraph::Paragraph;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

pub(super) fn compile_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
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
        throbber_state
            .get_mut()
            .calc_step(rand::random_range(0..Self::THROBBER_SET.symbols.len()) as i8);
        CompileCmd {
            running_duration: Duration::from_millis(0),
            compile_duration: Duration::from_millis(500),
            throbber_state,
            result: None,
        }
    }

    fn predefined_function_print(
        _meta: &mut CompiledProgram,
        _args: Vec<ProgramValue>,
    ) -> anyhow::Result<ProgramValue> {
        todo!()
    }

    fn predefined_functions() -> HashMap<&'static str, &'static PredefinedFunction<CompiledProgram>>
    {
        let (unlock_print, unlock_sleep, unlock_brk) = with_game_state(|game_state| {
            (
                game_state.upgrades.unlock_print.value(),
                game_state.upgrades.unlock_sleep.value(),
                game_state.upgrades.unlock_brk.value(),
            )
        });
        let mut functions = HashMap::new();

        if unlock_print {
            functions.insert(
                "print_function",
                &Self::predefined_function_print as &'static PredefinedFunction<CompiledProgram>,
            );
        }

        if unlock_sleep {
            todo!()
        }

        if unlock_brk {
            todo!()
        }

        functions
    }

    fn compile_result() -> anyhow::Result<()> {
        let parse_result_run_result = with_game_state(|game_state| -> anyhow::Result<_> {
            let parsed = parse_program(&game_state.program_code);
            match parsed {
                Ok(parsed) => {
                    let mut compiled = CompiledProgram::new();
                    let run_result =
                        compile_with_meta(&parsed, Self::predefined_functions(), &mut compiled);
                    Ok(match run_result {
                        Ok(()) => Ok(compiled),
                        Err(e) => Err((e.to_string(), compiled.instruction_counts)),
                    })
                }
                Err(richs) => Err(anyhow!(
                    richs
                        .into_iter()
                        .map(|rich| format!("{rich}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                )),
            }
        });
        let run_result = parse_result_run_result?;
        with_game_state_mut(|game_state| {
            game_state.compiled_program = Some(run_result);
        });
        Ok(())
    }
}

impl RunningCommand<SceneSwitch> for CompileCmd {
    fn is_done(&self) -> bool {
        self.result.is_some()
    }

    fn update(&mut self, _events: &[Event], time_delta: Duration) {
        if self.compile_duration <= self.running_duration {
            if self.result.is_none() {
                // TODO: run this while actually waiting, not just at the end
                self.result = Some(Self::compile_result());
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

use crate::backend::events::Event;
use crate::game_scenes::base::SceneSwitch;
use crate::game_state::{CompiledProgram, GameState, with_game_state, with_game_state_mut};
use crate::widgets::terminal::{ChainCmd, ParagraphCmd, RunningCommand};
use anyhow::anyhow;
use language::{PredefinedFunction, compile_with_meta, parse_program};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::text::Text;
use ratatui_core::widgets::StatefulWidget;
use ratatui_widgets::paragraph::Paragraph;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

pub(super) fn run_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    match with_game_state(|game_state| game_state.compiled_program.clone()) {
        None => Box::new(ParagraphCmd::new(Paragraph::new(Text::from(
            "The current code has not been compiled yet.",
        )))),
        Some(result) => match result {
            Err(err) => Box::new(ParagraphCmd::new(Paragraph::new(Text::from(err.clone())))),
            Ok(compiled_program) => Box::new(ChainCmd::new(
                Box::new(RunCmd::new(compiled_program.execution_time())),
                Box::new(move |run_cmd| {
                    let result = run_cmd.result.as_ref().expect("run command to finish");
                    let text = if let Err(e) = result {
                        Text::from(e.to_string())
                    } else {
                        let resource_gain = compiled_program.resource_gain();
                        with_game_state_mut(|game_state| {
                            game_state.current_resources += resource_gain.clone()
                        });
                        Text::from(format!("Gained {}", resource_gain.fmt_oneline()))
                    };
                    Box::new(ParagraphCmd::new(Paragraph::new(text)))
                }),
                true,
            )),
        },
    }
}

struct RunCmd {
    // when waiting
    active_duration: Duration,
    completion_duration: Duration,
    throbber_state: RefCell<throbber_widgets_tui::ThrobberState>,
    // after waiting
    result: Option<anyhow::Result<()>>,
}

impl RunCmd {
    const THROBBER_STEP_SPEED: Duration = Duration::from_millis(300);
    const THROBBER_SET: throbber_widgets_tui::Set = throbber_widgets_tui::BRAILLE_SIX;

    fn new(duration: Duration) -> Self {
        let mut throbber_state = RefCell::new(throbber_widgets_tui::ThrobberState::default());
        throbber_state.get_mut().calc_step(0); // randomize animation start
        RunCmd {
            active_duration: Duration::from_millis(0),
            completion_duration: duration,
            throbber_state,
            result: None,
        }
    }

    fn get_predefined_functions() -> HashMap<&'static str, &'static PredefinedFunction> {
        HashMap::new() // TODO
    }

    fn compile_result(game_state: &mut GameState) -> anyhow::Result<()> {
        let parsed = parse_program(&game_state.program_code);
        match parsed {
            Ok(parsed) => {
                let mut compiled = CompiledProgram::new();
                let run_result =
                    compile_with_meta(&parsed, Self::get_predefined_functions(), &mut compiled);
                game_state.compiled_program = Some(match run_result {
                    Ok(()) => Ok(compiled),
                    Err(e) => Err(e.to_string()),
                });
                Ok(())
            }
            Err(richs) => {
                let errs = richs.into_iter().map(|rich| Err(anyhow!("{rich}")));
                errs.collect()
            }
        }
    }
}

impl RunningCommand<SceneSwitch> for RunCmd {
    fn is_done(&self) -> bool {
        self.result.is_some()
    }

    fn update(&mut self, _events: &[Event], time_delta: Duration) {
        if self.completion_duration <= self.active_duration {
            if self.result.is_none() {
                // TODO: run this while actually waiting, not just at the end
                self.result = Some(with_game_state_mut(|game_state| {
                    Self::compile_result(game_state)
                }));
            }
        } else {
            // Animate loading
            let throbber_animation_steps =
                |d: Duration| d.div_duration_f32(RunCmd::THROBBER_STEP_SPEED) as i8;
            let old_duration = self.active_duration;
            self.active_duration += time_delta;
            let throbber_animation_step_div = throbber_animation_steps(self.active_duration)
                - throbber_animation_steps(old_duration);
            if throbber_animation_step_div > 0 {
                self.throbber_state
                    .borrow_mut()
                    .calc_step(throbber_animation_step_div);
            }
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let full = throbber_widgets_tui::Throbber::default()
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
            .throbber_style(
                ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD),
            )
            .throbber_set(RunCmd::THROBBER_SET)
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

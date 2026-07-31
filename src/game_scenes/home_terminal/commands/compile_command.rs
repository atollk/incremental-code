use crate::backend::events::Event;
use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::logic::auto_run::with_auto_run_mut;
use crate::game_scenes::logic::compilation::compile_thread;
use crate::game_state::with_game_state;
use crate::widgets::terminal::{ChainCmd, ParagraphCmd, RunningCommand};
use anyhow::anyhow;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::text::Text;
use ratatui_core::widgets::StatefulWidget;
use ratatui_widgets::paragraph::Paragraph;
use std::cell::RefCell;
use std::time::Duration;

pub(super) fn compile_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    let program_is_empty = with_game_state(|game_state| game_state.program_code.is_empty());
    if program_is_empty {
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
                    .expect("compile command did not finish");
                let paragraph: Paragraph<'static> = if let Err(e) = result {
                    Paragraph::new(e.to_string())
                } else {
                    Paragraph::new("Compilation successful.")
                };
                with_auto_run_mut(|auto_run| {
                    auto_run.reset();
                });
                Box::new(ParagraphCmd::new(paragraph))
            }),
            true,
        ))
    }
}

struct CompileCmd {
    // when waiting
    running_duration: Duration,
    compile_duration: Duration,
    throbber_state: RefCell<throbber_widgets_tui::ThrobberState>,
    cancelled: bool,
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
        compile_thread::with_compile_thread_mut(|compile_thread| compile_thread.compile());
        let compile_duration = Duration::from_secs_f32(with_game_state(|game_state| {
            game_state.upgrades.compile_time.value()
        }));
        CompileCmd {
            running_duration: Duration::from_millis(0),
            compile_duration,
            throbber_state,
            cancelled: false,
            result: None,
        }
    }
}

impl RunningCommand<SceneSwitch> for CompileCmd {
    fn is_done(&self) -> bool {
        self.result.is_some()
    }

    fn update(&mut self, _events: &[Event], time_delta: Duration) {
        if self.cancelled {
            if self.result.is_none() {
                self.result = Some(Err(anyhow!("Compilation cancelled")));
            }
            return;
        }
        let waiting_for_compilation = self.compile_duration > self.running_duration
            || compile_thread::with_compile_thread(|compile_thread| {
                !matches!(
                    compile_thread.status(),
                    compile_thread::CompileThreadStatus::Idle(_)
                )
            });
        if waiting_for_compilation {
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
        } else {
            if self.result.is_none() {
                if let compile_thread::CompileThreadStatus::Idle(result) =
                    compile_thread::with_compile_thread(|compile_thread| compile_thread.status())
                {
                    self.result = Some(result.map_err(|s| anyhow!(s)));
                } else {
                    unreachable!();
                }
            }
        }
    }

    fn cancel(&mut self) {
        self.cancelled = true;
        compile_thread::with_compile_thread_mut(|compile_thread| compile_thread.cancel());
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
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

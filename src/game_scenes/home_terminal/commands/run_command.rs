use crate::game_scenes::base::SceneSwitch;
use crate::game_state::with_game_state;
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use ratatui_core::text::Text;
use ratatui_widgets::paragraph::Paragraph;

pub(super) fn run_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    let text = match with_game_state(|game_state| game_state.compiled_program.clone()) {
        None => Text::from("The current code has not been compiled yet."),
        Some(result) => {
            // TODO: wait for execution time
            match result {
                Err(err) => Text::from(err.clone()),
                Ok(compiled_program) => {
                    let resource_gain = compiled_program.resource_gain();
                    with_game_state(|game_state| {
                        game_state.current_resources += resource_gain.clone()
                    });
                    Text::from(format!("Gained {}", resource_gain.fmt_oneline()))
                }
            }
        }
    };
    Box::new(ParagraphCmd::new(Paragraph::new(text)))
}

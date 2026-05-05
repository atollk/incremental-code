use crate::backend::events::Event;
use crate::basic_terminal_app::App;
use crate::game_state::{AUTO_SAVER, load_game_state};
use std::ops::{ControlFlow, FromResidual, Residual, Try};

/// A game scene that renders itself and handles input each frame.
pub trait Scene {
    /// Called once per frame with the accumulated events, the current ratatui frame, and the
    /// time elapsed since the previous frame.
    ///
    /// Returns a [`SceneSwitch`] that controls whether the scene stays active, exits the game,
    /// or hands off to a different scene.
    fn frame(
        &mut self,
        events: &[Event],
        frame: &mut ratatui_core::terminal::Frame,
        time_delta: web_time::Duration,
    ) -> SceneSwitch;
}

/// Returned by [`Scene::frame`] to tell the scene manager what to do next.
pub enum SceneSwitch {
    NoSwitch,
    ExitGame,
    SwitchTo(Box<dyn Scene>),
}

impl Residual<()> for SceneSwitch {
    type TryType = SceneSwitch;
}

impl FromResidual for SceneSwitch {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        residual
    }
}

impl Try for SceneSwitch {
    type Output = ();
    type Residual = SceneSwitch;

    fn from_output(_output: Self::Output) -> Self {
        SceneSwitch::NoSwitch
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        if let SceneSwitch::NoSwitch = self {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(self)
        }
    }
}

impl Default for SceneSwitch {
    fn default() -> Self {
        Self::NoSwitch
    }
}

/// Root [`App`](App) that owns the active scene and tracks frame timing.
pub struct SceneGame {
    active_scene: Box<dyn Scene>,
    last_frame: web_time::Instant,
}

impl SceneGame {
    /// Creates a `SceneGame` starting with the given initial scene.
    pub fn new(scene: Box<dyn Scene>) -> Self {
        if let Err(e) = load_game_state() {
            log::error!("{e}");
        }
        AUTO_SAVER
            .lock()
            .unwrap()
            .start(std::time::Duration::from_secs(60));
        SceneGame {
            active_scene: scene,
            last_frame: web_time::Instant::now(),
        }
    }
}

impl App for SceneGame {
    fn frame(
        &mut self,
        events: &[Event],
        frame: &mut ratatui_core::terminal::Frame,
    ) -> anyhow::Result<bool> {
        let elapsed = web_time::Instant::now() - self.last_frame;
        self.last_frame = web_time::Instant::now();
        let scene_switch = self.active_scene.frame(events, frame, elapsed);
        match scene_switch {
            SceneSwitch::NoSwitch => {}
            SceneSwitch::ExitGame => {
                return Ok(true);
            }
            SceneSwitch::SwitchTo(new_scene) => {
                self.active_scene = new_scene;
            }
        }
        Ok(false)
    }
}

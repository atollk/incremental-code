use crate::backend::events::Event;
use crate::basic_terminal_app::App;
use std::ops::{ControlFlow, FromResidual, Residual, Try};

pub trait Scene {
    fn frame(
        &mut self,
        events: &[Event],
        frame: &mut ratatui_core::terminal::Frame,
        time_delta: web_time::Duration,
    ) -> SceneSwitch;
}

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

pub struct SceneGame {
    active_scene: Box<dyn Scene>,
    last_frame: web_time::Instant,
}

impl SceneGame {
    pub fn new(scene: Box<dyn Scene>) -> Self {
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

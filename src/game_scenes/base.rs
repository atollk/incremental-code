use crate::backend::events::{Event, MetaEvent};
use crate::basic_terminal_app::App;
use crate::game_scenes::logic::audio::GLOBAL_AUDIO_BACKEND;
use crate::game_scenes::logic::auto_run::GLOBAL_AUTO_RUN;
use crate::game_state::GLOBAL_AUTO_SAVER;
use parking_lot::ReentrantMutex;
use std::cell::RefCell;
use std::ops::{ControlFlow, FromResidual, Residual, Try};
use std::time::Duration;

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
    tickers: Vec<&'static dyn Ticker>,
}

pub trait Ticker {
    fn tick(&self, elapsed: Duration);
}

pub trait TickerMut {
    fn tick(&mut self, elapsed: Duration);
}

impl<T: Ticker> TickerMut for T {
    fn tick(&mut self, elapsed: Duration) {
        Ticker::tick(self, elapsed);
    }
}

impl<T: TickerMut> Ticker for ReentrantMutex<RefCell<T>> {
    fn tick(&self, elapsed: Duration) {
        self.lock().borrow_mut().tick(elapsed);
    }
}

impl SceneGame {
    /// Creates a `SceneGame` starting with the given initial scene.
    pub fn new(scene: Box<dyn Scene>) -> Self {
        let mut scene_game = SceneGame {
            active_scene: scene,
            last_frame: web_time::Instant::now(),
            tickers: Vec::new(),
        };
        scene_game.add_ticker(&*GLOBAL_AUTO_SAVER);
        scene_game.add_ticker(&*GLOBAL_AUDIO_BACKEND);
        scene_game.add_ticker(&*GLOBAL_AUTO_RUN);
        scene_game
    }

    fn add_ticker(&mut self, ticker: &'static dyn Ticker) {
        self.tickers.push(ticker);
    }
}

impl App for SceneGame {
    fn frame(
        &mut self,
        events: &[Event],
        frame: &mut ratatui_core::terminal::Frame,
    ) -> anyhow::Result<bool> {
        let elapsed = web_time::Instant::now() - self.last_frame;

        for ticker in &self.tickers {
            ticker.tick(elapsed);
        }

        for event in events {
            if let Event::MetaEvent(MetaEvent::SigTerm) = event {
                return Ok(true);
            }
        }

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

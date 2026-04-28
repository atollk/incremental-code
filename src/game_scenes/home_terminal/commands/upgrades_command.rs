use crate::backend::events::Event;
use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::upgrades::UpgradesScene;
use crate::widgets::terminal::RunningCommand;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use std::time::Duration;

pub(super) fn upgrades_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    Box::new(UpgradesCmd {})
}

struct UpgradesCmd {}

impl RunningCommand<SceneSwitch> for UpgradesCmd {
    fn is_done(&self) -> bool {
        true
    }

    fn update(&mut self, _events: &[Event], _time_delta: Duration) {}

    fn render(&self, _area: Rect, _buf: &mut Buffer) {}

    fn height(&self, _columns: u16) -> u16 {
        0
    }

    fn get_metadata(&self) -> SceneSwitch {
        SceneSwitch::SwitchTo(Box::new(UpgradesScene::default()))
    }
}

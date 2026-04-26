use crate::backend::events::Event;
use crate::game_scenes::base::{Scene, SceneSwitch};
use crate::widgets::tree::{Tree, TreeItem, TreeState};
use ouroboros::self_referencing;
use ratatui_core::terminal::Frame;
use std::time::Duration;

pub struct UpgradesScene<'a> {
    tree_widget: TreeWidget<'a>,
}

#[self_referencing]
struct TreeWidget<'a> {
    tree_items: Vec<TreeItem<'a, &'a str>>,
    #[borrows(tree_items)]
    #[covariant]
    tree: Tree<'this, &'a str>,
    tree_state: TreeState<&'a str>,
}

impl<'a> Default for UpgradesScene<'a> {
    fn default() -> Self {
        let tree_items = vec![TreeItem::new("foo", "foo text", vec![]).unwrap()];
        let tree_widget = TreeWidget::new(
            tree_items,
            |tree_items| Tree::new(tree_items).unwrap(),
            TreeState::default(),
        );
        UpgradesScene { tree_widget }
    }
}

impl<'a> Scene for UpgradesScene<'a> {
    fn frame(&mut self, events: &[Event], frame: &mut Frame, time_delta: Duration) -> SceneSwitch {
        self.tree_widget.with_mut(|tree| {
            frame.render_stateful_widget(&*tree.tree, frame.area(), tree.tree_state)
        });
        SceneSwitch::NoSwitch
    }
}

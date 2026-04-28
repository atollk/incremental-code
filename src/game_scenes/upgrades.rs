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
    tree_items: Vec<TreeItem<'a, u64>>,
    #[borrows(tree_items)]
    #[covariant]
    tree: Tree<'this, u64>,
    tree_state: TreeState<u64>,
}

impl<'a> UpgradesScene<'a> {
    fn build_tree_items() -> Vec<TreeItem<'a, u64>> {
        vec![
            TreeItem::new(
                1,
                "foo text",
                vec![TreeItem::new(2, "x text", vec![]).unwrap()],
            )
            .unwrap(),
            TreeItem::new(3, "bar text", vec![]).unwrap(),
        ]
    }
}

impl<'a> Default for UpgradesScene<'a> {
    fn default() -> Self {
        let tree_widget = TreeWidget::new(
            Self::build_tree_items(),
            |tree_items| Tree::new(tree_items).unwrap(),
            TreeState::default(),
        );
        UpgradesScene { tree_widget }
    }
}

impl<'a> Scene for UpgradesScene<'a> {
    fn frame(&mut self, events: &[Event], frame: &mut Frame, time_delta: Duration) -> SceneSwitch {
        self.tree_widget.with_tree_state_mut(|tree_state| {
            for event in events {
                tree_state.process_input_event(event);
            }
        });
        self.tree_widget.with_mut(|tree| {
            frame.render_stateful_widget(&*tree.tree, frame.area(), tree.tree_state)
        });
        SceneSwitch::NoSwitch
    }
}

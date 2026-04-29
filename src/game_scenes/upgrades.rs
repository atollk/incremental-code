use crate::backend::events::Event;
use crate::game_scenes::base::{Scene, SceneSwitch};
use crate::game_state::{Resources, Upgrade, UpgradeCollection, Upgrades, with_game_state};
use crate::widgets::tree::{Tree, TreeItem, TreeState};
use itertools::Itertools;
use ouroboros::self_referencing;
use ratatui_core::style::{Modifier, Style};
use ratatui_core::terminal::Frame;
use ratatui_core::text::Line;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

pub struct UpgradesScene<'a> {
    tree_widget: TreeWidget<'a>,
    upgrades_working_copy: Upgrades,
}

#[self_referencing]
struct TreeWidget<'a> {
    tree_items: Vec<TreeItem<'a, u64>>,
    #[borrows(tree_items)]
    #[covariant]
    tree: Tree<'this, u64>,
    tree_state: TreeState<u64>,
}

fn render_upgrade(upgrade: &dyn Upgrade) -> Line<'static> {
    Line::raw(format!(
        "{}    {}/{}    {}",
        upgrade.name(),
        upgrade.current_level(),
        upgrade.max_level(),
        upgrade.next_level_cost().unwrap_or(Resources::default()),
    ))
}

impl<'a> UpgradesScene<'a> {
    fn build_tree_items(upgrades: &Upgrades) -> Vec<TreeItem<'a, u64>> {
        let upgrade_line_l1 = upgrades.level1.upgrades().map(render_upgrade).collect_vec();
        vec![
            TreeItem::new(
                1,
                "Level 1 upgrades".to_string(),
                upgrade_line_l1
                    .into_iter()
                    .map(|line| {
                        let hash = {
                            let mut s = DefaultHasher::new();
                            line.hash(&mut s);
                            s.finish()
                        };
                        TreeItem::new_leaf(hash, line)
                    })
                    .collect(),
            )
            .unwrap(),
        ]
    }
}

impl<'a> Default for UpgradesScene<'a> {
    fn default() -> Self {
        let upgrades = with_game_state(|game_state| game_state.upgrades.clone());
        let tree_widget = TreeWidget::new(
            Self::build_tree_items(&upgrades),
            |tree_items| {
                Tree::new(tree_items)
                    .unwrap()
                    .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            },
            TreeState::default(),
        );
        UpgradesScene {
            tree_widget,
            upgrades_working_copy: upgrades,
        }
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

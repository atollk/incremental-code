use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind, MouseEventKind};
use crate::game_scenes::base::{Scene, SceneSwitch};
use crate::game_state::{Upgrade, UpgradeCollection, Upgrades, with_game_state};
use crate::widgets::tree::{Tree, TreeItem, TreeState};
use itertools::Itertools;
use ouroboros::self_referencing;
use ratatui_core::layout::Position;
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

fn find_item_in_tree<'a, 'b>(
    tree_items: &'b [TreeItem<'a, u64>],
    identifier_path: &[u64],
) -> Option<&'b TreeItem<'a, u64>> {
    if let [head, tail @ ..] = identifier_path {
        tree_items
            .iter()
            .find(|child| child.identifier() == head)
            .and_then(|child| child.find_child(tail))
    } else {
        None
    }
}

const EMPTY_BOX: char = '🔲';
const FULL_BOX: char = '⬛';

fn render_upgrade(upgrade: &dyn Upgrade, name_width: usize, level_width: usize) -> Line<'static> {
    let level_str = format!(
        "{}{}",
        std::iter::repeat(EMPTY_BOX)
            .take(upgrade.current_level() as usize)
            .collect::<String>(),
        std::iter::repeat(FULL_BOX)
            .take((upgrade.max_level() - upgrade.current_level()) as usize)
            .collect::<String>(),
    );
    let cost_str = match upgrade.next_level_cost() {
        Some(r) => r.to_string(),
        None => "maxed".to_string(),
    };
    Line::raw(format!(
        "{:<name_width$}  {:>level_width$}  {}",
        upgrade.name(),
        level_str,
        cost_str,
    ))
}

impl<'a> UpgradesScene<'a> {
    fn build_tree_items(upgrades: &Upgrades) -> Vec<TreeItem<'a, u64>> {
        let upgrades_l1: Vec<&dyn Upgrade> = upgrades.level1.upgrades().collect_vec();
        let name_width = upgrades_l1
            .iter()
            .map(|u| u.name().len())
            .max()
            .unwrap_or(0);
        let level_width = upgrades_l1
            .iter()
            .map(|u| format!("{}/{}", u.current_level(), u.max_level()).len())
            .max()
            .unwrap_or(0);
        let upgrade_line_l1 = upgrades_l1
            .iter()
            .map(|u| render_upgrade(*u, name_width, level_width))
            .collect_vec();
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

    fn process_input_event(&mut self, event: &Event) {
        self.tree_widget.with_mut(|tree_widget| {
            match event {
                Event::KeyEvent(key) => {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                tree_widget.tree_state.toggle_selected();
                            }
                            KeyCode::Left => {
                                if tree_widget.tree_state.selected().is_empty() {
                                    // noop
                                } else {
                                    let item = find_item_in_tree(&tree_widget.tree_items, &tree_widget.tree_state.selected()).expect("when a tree item is selected, you should be able to find it via its identifier");
                                    if item.children().is_empty() {
                                        // no children on the selected item -> level down
                                        todo!()
                                    } else {
                                        // children -> close it
                                        tree_widget.tree_state.key_left();
                                    }
                                }
                            }
                            KeyCode::Right => {
                                if tree_widget.tree_state.selected().is_empty() {
                                    // noop
                                } else {
                                    let item = find_item_in_tree(&tree_widget.tree_items, &tree_widget.tree_state.selected()).expect("when a tree item is selected, you should be able to find it via its identifier");
                                    if item.children().is_empty() {
                                        // no children on the selected item -> level up
                                        todo!()
                                    } else {
                                        // children -> open it
                                        tree_widget.tree_state.key_right();
                                    }
                                }
                            }
                            KeyCode::Down => {
                                tree_widget.tree_state.key_down();
                            }
                            KeyCode::Up => {
                                tree_widget.tree_state.key_up();
                            }
                            KeyCode::Esc => {
                                tree_widget.tree_state.select(Vec::new());
                            }
                            KeyCode::PageDown => {
                                tree_widget.tree_state.scroll_down(3);
                            }
                            KeyCode::PageUp => {
                                tree_widget.tree_state.scroll_up(3);
                            }
                            _ => (),
                        }
                    }
                }
                Event::MouseEvent(mouse) => match mouse.kind {
                    MouseEventKind::ScrollDown => {
                        tree_widget.tree_state.scroll_down(1);
                    }
                    MouseEventKind::ScrollUp => {
                        tree_widget.tree_state.scroll_up(1);
                    }
                    MouseEventKind::Down(_button) => {
                        tree_widget.tree_state.click_at(Position::new(mouse.column, mouse.row));
                    }
                    _ => (),
                },
            };
        });
    }
}

impl<'a> Default for UpgradesScene<'a> {
    fn default() -> Self {
        let upgrades = with_game_state(|game_state| game_state.upgrades.clone());
        let mut tree_widget = TreeWidget::new(
            Self::build_tree_items(&upgrades),
            |tree_items| {
                Tree::new(tree_items)
                    .unwrap()
                    .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            },
            TreeState::default(),
        );
        tree_widget.with_tree_state_mut(|state| state.select_first());
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

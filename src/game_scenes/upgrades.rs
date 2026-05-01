use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind, MouseEventKind};
use crate::game_scenes::base::{Scene, SceneSwitch};
use crate::game_scenes::home_terminal::HomeTerminalScene;
use crate::game_state::{Resources, Upgrade, UpgradeCollection, Upgrades, with_game_state};
use crate::widgets::dialog::{ConfirmDialog, ConfirmResult};
use crate::widgets::hud::hud_layout;
use crate::widgets::tree::{Tree, TreeItem, TreeState};
use itertools::Itertools;
use ouroboros::self_referencing;
use ratatui_core::layout::Position;
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::terminal::Frame;
use ratatui_core::text::{Line, Span};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct UpgradesScene<'a> {
    tree_widget: TreeWidget<'a>,
    confirm_dialog: Option<ConfirmDialog>,
    upgrades_working_copy: Upgrades,
    resources_backup: (Resources, Resources),
}

#[self_referencing]
struct TreeWidget<'a> {
    tree_items: Vec<TreeItem<'a, u64>>,
    #[borrows(tree_items)]
    #[covariant]
    tree: Tree<'this, u64>,
    tree_state: TreeState<u64>,
}

impl<'a> Default for UpgradesScene<'a> {
    fn default() -> Self {
        let (upgrades, current_resources, carryover_resources) = with_game_state(|game_state| {
            (
                game_state.upgrades.clone(),
                game_state.current_resources.clone(),
                game_state.carryover_resources.clone(),
            )
        });
        let mut tree_widget = Self::create_tree_widget(&upgrades);
        tree_widget.with_tree_state_mut(|state| state.select(vec![1]));
        UpgradesScene {
            tree_widget,
            confirm_dialog: None,
            upgrades_working_copy: upgrades,
            resources_backup: (current_resources, carryover_resources),
        }
    }
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

fn render_upgrade(upgrade: &dyn Upgrade, name_width: usize, level_width: usize) -> Line<'static> {
    const EMPTY_BOX: char = '🔲';
    const FULL_BOX: char = '⬛';
    let level_str = upgrade.format_level_str(EMPTY_BOX, FULL_BOX);
    let cost_str = upgrade.format_cost_str();
    let _current_value_str = upgrade.current_value_text();
    let _next_value_str = upgrade.next_value_text();
    Line::from_iter(vec![
        Span::raw(format!("{:<name_width$}", upgrade.name())),
        Span::raw("     "),
        Span::raw(format!(
            "{}{}",
            level_str,
            " ".repeat((level_width * FULL_BOX.width().unwrap()).saturating_sub(level_str.width()))
        )),
        Span::raw("     "),
        Span::raw("todo(value)"),
        Span::raw("     "),
        Span::styled(cost_str, {
            let cost = upgrade.next_level_cost();
            match cost {
                None => Style::default().fg(Color::Gray),
                Some(cost) => {
                    if cost <= with_game_state(|game_state| game_state.total_resources()) {
                        Style::default().fg(Color::White)
                    } else {
                        Style::default().fg(Color::LightRed)
                    }
                }
            }
        }),
    ])
}

fn hash_upgrade(upgrade: &dyn Upgrade) -> u64 {
    let mut s = DefaultHasher::new();
    upgrade.name().hash(&mut s);
    s.finish()
}

impl<'a> UpgradesScene<'a> {
    fn build_tree_items(upgrades: &Upgrades) -> Vec<TreeItem<'a, u64>> {
        let upgrades_l1: Vec<&dyn Upgrade> = upgrades.level1.upgrades().collect_vec();
        let name_width = upgrades_l1
            .iter()
            .map(|u| u.name().len())
            .max()
            .unwrap_or(0);
        let level_width = upgrades_l1.iter().map(|u| u.max_level()).max().unwrap_or(0);
        let level1 = TreeItem::new(
            1,
            "Level 1 upgrades".to_string(),
            upgrades_l1
                .into_iter()
                .map(|u| {
                    TreeItem::new_leaf(
                        hash_upgrade(u),
                        render_upgrade(u, name_width, level_width as usize),
                    )
                })
                .collect(),
        );
        vec![level1.unwrap()]
    }

    fn create_tree_widget(upgrades: &Upgrades) -> TreeWidget<'a> {
        TreeWidget::new(
            Self::build_tree_items(&upgrades),
            |tree_items| {
                Tree::new(tree_items)
                    .unwrap()
                    .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            },
            TreeState::default(),
        )
    }

    fn level(&mut self, identifier_path: &[u64], level_up: bool) {
        // Find the upgrade instance from the tree identifier
        let identifier_path: &[u64; 2] = identifier_path.try_into().unwrap();
        let upgrade_level = match identifier_path[0] {
            1 => &mut self.upgrades_working_copy.level1,
            _ => unreachable!(),
        };
        let (pos, _) = upgrade_level
            .upgrades()
            .find_position(|&u| hash_upgrade(u) == identifier_path[1])
            .expect("find the identifier from the hash");
        let upgrade = upgrade_level.upgrades_mut().nth(pos).unwrap();

        // Perform the leveling
        let refresh_required = if level_up {
            if let Some(cost) = upgrade.next_level_cost() {
                if with_game_state(|game_state| cost <= game_state.total_resources()) {
                    // can afford -> take resources and level up
                    with_game_state(|game_state| game_state.take_resources(&cost)).unwrap();
                    upgrade.level_up();
                    true
                } else {
                    // cannot afford -> do nothing
                    false
                }
            } else {
                // can't level up
                false
            }
        } else {
            if upgrade.current_level() == 0 {
                // can't level down
                false
            } else {
                // level down and return resources
                upgrade.level_down();
                let cost = upgrade
                    .next_level_cost()
                    .expect("After leveling down, a cost to level up should be defined.");
                // TODO: shortcut - we don't remember where resources came from, so we just return them as carryover
                with_game_state(|game_state| game_state.give_carryover_resources(&cost));
                true
            }
        };

        // Refresh visuals
        if refresh_required {
            let old_tree_state = self
                .tree_widget
                .with_tree_state(|tree_state| tree_state.clone());
            self.tree_widget = Self::create_tree_widget(&self.upgrades_working_copy);
            self.tree_widget
                .with_tree_state_mut(|new_tree_state| *new_tree_state = old_tree_state);
        }
    }

    #[must_use]
    fn process_input_event(&mut self, event: &Event) -> SceneSwitch {
        match event {
            Event::KeyEvent(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.tree_widget
                        .with_tree_state_mut(|ts| ts.toggle_selected());
                }
                KeyCode::Left => {
                    let selected = self
                        .tree_widget
                        .with_tree_state(|ts| ts.selected().to_vec());
                    if !selected.is_empty() {
                        let children_empty = self.tree_widget.with(|tw| {
                            find_item_in_tree(tw.tree_items, &selected)
                                .expect("when a tree item is selected, you should be able to find it via its identifier")
                                .children()
                                .is_empty()
                        });
                        if children_empty {
                            self.level(&selected, false);
                        } else {
                            self.tree_widget.with_tree_state_mut(|ts| ts.key_left());
                        }
                    }
                }
                KeyCode::Right => {
                    let selected = self
                        .tree_widget
                        .with_tree_state(|ts| ts.selected().to_vec());
                    if !selected.is_empty() {
                        let children_empty = self.tree_widget.with(|tw| {
                            find_item_in_tree(tw.tree_items, &selected)
                                .expect("when a tree item is selected, you should be able to find it via its identifier")
                                .children()
                                .is_empty()
                        });
                        if children_empty {
                            self.level(&selected, true);
                        } else {
                            self.tree_widget.with_tree_state_mut(|ts| ts.key_right());
                        }
                    }
                }
                KeyCode::Down => {
                    self.tree_widget.with_tree_state_mut(|ts| ts.key_down());
                }
                KeyCode::Up => {
                    self.tree_widget.with_tree_state_mut(|ts| ts.key_up());
                }
                KeyCode::Esc => {
                    if with_game_state(|game_state| {
                        self.upgrades_working_copy == game_state.upgrades
                    }) {
                        // no changes -> just switch back
                        return SceneSwitch::SwitchTo(Box::new(HomeTerminalScene::new()));
                    } else {
                        // upgrades leveled -> open confirm dialog
                        self.confirm_dialog = Some(ConfirmDialog::new(
                            "",
                            "Purchase selected upgrades? [Y]es  [N]o  [Esc] Cancel",
                        ));
                    }
                }
                KeyCode::PageDown => {
                    self.tree_widget.with_tree_state_mut(|ts| ts.scroll_down(3));
                }
                KeyCode::PageUp => {
                    self.tree_widget.with_tree_state_mut(|ts| ts.scroll_up(3));
                }
                _ => {}
            },
            Event::MouseEvent(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => {
                    self.tree_widget.with_tree_state_mut(|ts| ts.scroll_down(1));
                }
                MouseEventKind::ScrollUp => {
                    self.tree_widget.with_tree_state_mut(|ts| ts.scroll_up(1));
                }
                MouseEventKind::Down(_button) => {
                    self.tree_widget.with_tree_state_mut(|ts| {
                        ts.click_at(Position::new(mouse.column, mouse.row))
                    });
                }
                _ => {}
            },
            _ => {}
        }
        SceneSwitch::NoSwitch
    }
}

impl<'a> Scene for UpgradesScene<'a> {
    fn frame(&mut self, events: &[Event], frame: &mut Frame, _time_delta: Duration) -> SceneSwitch {
        // ConfirmingExit mode: dialog is active
        if self.confirm_dialog.is_some() {
            for event in events {
                self.confirm_dialog.as_mut().unwrap().handle_event(event);
            }
            let result = self.confirm_dialog.as_ref().unwrap().result();
            let dialog_scene_switch = match result {
                Some(ConfirmResult::Yes) => {
                    with_game_state(|game_state| {
                        game_state.upgrades = self.upgrades_working_copy.clone()
                    });
                    SceneSwitch::SwitchTo(Box::new(HomeTerminalScene::new()))
                }
                Some(ConfirmResult::No) => {
                    with_game_state(|game_state| {
                        game_state.current_resources = self.resources_backup.0.clone();
                        game_state.carryover_resources = self.resources_backup.1.clone();
                    });
                    SceneSwitch::SwitchTo(Box::new(HomeTerminalScene::new()))
                }
                Some(ConfirmResult::Cancel) => {
                    self.confirm_dialog = None;
                    SceneSwitch::NoSwitch
                }
                None => SceneSwitch::NoSwitch,
            };

            let content_area = hud_layout(frame);
            self.tree_widget.with_mut(|tree| {
                frame.render_stateful_widget(&*tree.tree, content_area, tree.tree_state)
            });

            if let Some(dialog) = &self.confirm_dialog {
                frame.render_widget(dialog, frame.area());
            }
            dialog_scene_switch?;
        }

        // Upgrade screen
        for event in events {
            self.process_input_event(event)?;
        }
        let content_area = hud_layout(frame);
        self.tree_widget.with_mut(|tree| {
            frame.render_stateful_widget(&*tree.tree, content_area, tree.tree_state)
        });
        SceneSwitch::NoSwitch
    }
}

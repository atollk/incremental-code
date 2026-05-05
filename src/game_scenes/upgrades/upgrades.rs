use crate::backend::audio::with_audio_backend;
use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind, MouseEventKind};
use crate::game_scenes::base::{Scene, SceneSwitch};
use crate::game_scenes::home_terminal::HomeTerminalScene;
use crate::game_scenes::upgrades::tree::{TreeWidget, create_tree_widget, find_item_in_tree};
use crate::game_state::{Resources, Upgrade, Upgrades, with_game_state, with_game_state_mut};
use crate::widgets::dialog::{ConfirmDialog, ConfirmResult};
use crate::widgets::hud::draw_hud;
use ratatui_core::layout::Position;
use ratatui_core::terminal::Frame;
use std::time::Duration;

pub struct UpgradesScene<'a> {
    tree_widget: TreeWidget<'a>,
    confirm_dialog: Option<ConfirmDialog>,
    upgrades_working_copy: Upgrades,
    resources_backup: (Resources, Resources),
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
        let mut tree_widget = create_tree_widget(&upgrades);
        tree_widget.with_tree_state_mut(|state| state.select(vec![0]));
        UpgradesScene {
            tree_widget,
            confirm_dialog: None,
            upgrades_working_copy: upgrades,
            resources_backup: (current_resources, carryover_resources),
        }
    }
}

impl<'a> UpgradesScene<'a> {
    fn level(&mut self, identifier_path: &[usize], level_up: bool) {
        // Find the upgrade instance from the tree identifier
        let identifier_path: &[usize; 2] = identifier_path.try_into().unwrap();
        let pos = self
            .upgrades_working_copy
            .upgrades()
            .into_iter()
            .enumerate()
            .filter(|(_, u)| u.group() == identifier_path[0])
            .nth(identifier_path[1] as usize)
            .unwrap_or_else(|| panic!("identifier_path out of bounds: {:?}", identifier_path))
            .0;
        let upgrade = self.upgrades_working_copy.upgrade_at_mut(pos);

        // Perform the leveling
        let refresh_required = if level_up {
            if let Some(cost) = upgrade.next_level_cost() {
                if with_game_state(|game_state| cost <= game_state.total_resources()) {
                    // can afford -> take resources and level up
                    with_game_state_mut(|game_state| game_state.take_resources(&cost)).unwrap();
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
            if upgrade.get_level() == 0 {
                // can't level down
                false
            } else {
                // level down and return resources
                upgrade.level_down();
                let cost = upgrade
                    .next_level_cost()
                    .expect("After leveling down, a cost to level up should be defined.");
                // TODO: shortcut - we don't remember where resources came from, so we just return them as carryover
                with_game_state_mut(move |game_state| game_state.give_carryover_resources(cost));
                true
            }
        };

        // Refresh visuals
        if refresh_required {
            let old_tree_state = self
                .tree_widget
                .with_tree_state(|tree_state| tree_state.clone());
            self.tree_widget = create_tree_widget(&self.upgrades_working_copy);
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
                KeyCode::Left | KeyCode::Right => {
                    let selected = self
                        .tree_widget
                        .with_tree_state(|ts| ts.selected().to_vec());
                    if !selected.is_empty() {
                        let children_empty = self.tree_widget.with_tree_items(|tree_items| {
                            find_item_in_tree(tree_items, &selected)
                                .expect("when a tree item is selected, you should be able to find it via its identifier")
                                .children()
                                .is_empty()
                        });
                        if key.code == KeyCode::Left {
                            if children_empty {
                                self.level(&selected, false);
                            } else {
                                self.tree_widget.with_tree_state_mut(|ts| ts.key_left());
                            }
                        } else {
                            if children_empty {
                                self.level(&selected, true);
                            } else {
                                self.tree_widget.with_tree_state_mut(|ts| ts.key_right());
                            }
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
                    with_game_state_mut(|game_state| {
                        game_state.upgrades = self.upgrades_working_copy.clone()
                    });
                    on_upgrades_commit();
                    SceneSwitch::SwitchTo(Box::new(HomeTerminalScene::new()))
                }
                Some(ConfirmResult::No) => {
                    with_game_state_mut(|game_state| {
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

            let content_area =
                if with_game_state(|game_state| game_state.upgrades.unlock_hud.value()) {
                    draw_hud(frame)
                } else {
                    frame.area()
                };
            frame.render_widget(&mut self.tree_widget, content_area);

            if let Some(dialog) = &self.confirm_dialog {
                frame.render_widget(dialog, frame.area());
            }
            dialog_scene_switch?;
        }

        // Upgrade screen
        for event in events {
            self.process_input_event(event)?;
        }
        let content_area = if with_game_state(|game_state| game_state.upgrades.unlock_hud.value()) {
            draw_hud(frame)
        } else {
            frame.area()
        };
        frame.render_widget(&mut self.tree_widget, content_area);
        SceneSwitch::NoSwitch
    }
}

fn on_upgrades_commit() {
    let unlock_music = with_game_state(|game_state| game_state.upgrades.unlock_music.value());
    if unlock_music {
        with_audio_backend(|audio| {
            audio
                .start_bgm()
                .map_err(|e| log::warn!("Error starting bgm: {}", e))
        });
    }
}

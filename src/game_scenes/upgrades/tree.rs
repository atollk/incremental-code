use crate::game_state::{Resources, Upgrade, Upgrades, with_game_state};
use crate::widgets::tree::{Tree, TreeItem, TreeState};
use itertools::Itertools;
use ouroboros::self_referencing;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::text::{Line, Span};
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_widgets::scrollbar::Scrollbar;
use std::cmp::max;
use std::ops::Deref;

#[self_referencing]
pub(super) struct TreeWidget<'a> {
    pub tree_items: Vec<TreeItem<'a, usize>>,
    #[borrows(tree_items)]
    #[covariant]
    pub tree: Tree<'this, usize>,
    pub tree_state: TreeState<usize>,
}

struct TreeColumns<T> {
    name: T,
    level: T,
    level_up_cost: T,
    values: T,
}

fn render_column_texts(upgrade: &dyn Upgrade) -> TreeColumns<String> {
    let current_value_str = upgrade.value_text();
    let next_value_str = upgrade
        .next_level_value_text()
        .map(|s| s.to_string())
        .unwrap_or_default();

    TreeColumns {
        name: upgrade.name().to_string(),
        level: String::new(),
        level_up_cost: String::new(),
        values: match (current_value_str.deref(), next_value_str.as_str()) {
            ("", "") => "".to_string(),
            ("", _) => format!("-> {}", next_value_str),
            (_, "") => current_value_str.to_string(),
            (_, _) => format!("{} -> {}", current_value_str, next_value_str),
        },
    }
}

/// Builds colored spans for a cost, one per non-zero denomination (mirroring the
/// cascade in `Resources::fmt_oneline`), coloring only the denominations the
/// player is actually short on in red.
fn cost_spans(cost: &Resources, available: &Resources) -> Vec<Span<'static>> {
    let write_stars = cost.stars.0 != 0.0;
    let write_diamond = cost.diamond.0 != 0.0 || write_stars;
    let write_gold = cost.gold.0 != 0.0 || write_diamond;
    let write_silver = cost.silver.0 != 0.0 || write_gold;
    let denoms = [
        (write_stars, cost.stars, available.stars, '⭐'),
        (write_diamond, cost.diamond, available.diamond, '💎'),
        (write_gold, cost.gold, available.gold, '🟡'),
        (write_silver, cost.silver, available.silver, '⚪'),
        (true, cost.bronze, available.bronze, '🟤'), // bronze always shown
    ];
    let mut spans = Vec::new();
    for (write, amount, avail, symbol) in denoms {
        if !write {
            continue;
        }
        if !spans.is_empty() {
            spans.push(Span::raw(" "));
        }
        let color = if amount.0 > avail.0 {
            Color::LightRed
        } else {
            Color::White
        };
        spans.push(Span::styled(
            format!("{amount} {symbol}"),
            Style::default().fg(color),
        ));
    }
    spans
}

fn render_track_items(upgrade: &dyn Upgrade) -> Vec<TreeItem<'static, usize>> {
    (0..upgrade.count_tracks())
        .filter(|&track| upgrade.track_next_cost(track).is_some())
        .map(|track| {
            let track_level = upgrade.track_get_level(track);
            let max_level = upgrade.max_level();
            let cost = upgrade.track_next_cost(track);
            let mut spans = vec![Span::raw(format!(
                "-  {:>3}/{:<3}  ",
                track_level, max_level
            ))];
            match &cost {
                None => spans.push(Span::styled("maxed", Style::default().fg(Color::Gray))),
                Some(c) => {
                    let available = with_game_state(|game_state| game_state.total_resources());
                    spans.extend(cost_spans(c, &available));
                }
            }
            TreeItem::new_leaf(track, Line::from(spans))
        })
        .collect()
}

fn render_group_items(upgrades: &[&dyn Upgrade], group_i: usize) -> Vec<TreeItem<'static, usize>> {
    let group_items = upgrades.iter().filter(|u| u.group() == group_i);
    let group_item_strings: Vec<(&dyn Upgrade, TreeColumns<String>)> = group_items
        .map(|&u| (u, render_column_texts(u)))
        .collect_vec();

    // For each column, find the longest text
    let column_sizes = group_item_strings
        .iter()
        .map(|(_, tc)| TreeColumns {
            name: tc.name.len(),
            level: tc.level.len(),
            level_up_cost: tc.level_up_cost.len(),
            values: tc.values.len(),
        })
        .fold(
            TreeColumns {
                name: 0,
                level: 0,
                level_up_cost: 0,
                values: 0,
            },
            |acc, u| TreeColumns {
                name: max(acc.name, u.name),
                level: max(acc.level, u.level),
                level_up_cost: max(acc.level_up_cost, u.level_up_cost),
                values: max(acc.values, u.values),
            },
        );

    group_item_strings
        .into_iter()
        .enumerate()
        .map(|(i, (u, tc))| {
            let name_width = column_sizes.name;
            let values_width = column_sizes.values;
            let spans = vec![
                Span::raw(format!("{:<name_width$}", tc.name)),
                Span::raw("    "),
                Span::raw(format!("{:^values_width$}", tc.values)),
            ];
            let track_children = render_track_items(u);
            TreeItem::new(i, Line::from_iter(spans), track_children).unwrap()
        })
        .collect()
}

fn groups_are_unlocked(upgrades: &Upgrades) -> [bool; 7] {
    [
        true,
        upgrades.unlock_level1.value(),
        upgrades.unlock_level2.value(),
        upgrades.unlock_level3.value(),
        upgrades.unlock_level4.value(),
        upgrades.unlock_level5.value(),
        upgrades.unlock_level6.value(),
    ]
}

const GROUP_FINISHED_ICON: &str = "✅";

/// A group is "finished" once every upgrade in it has no purchasable track left.
fn group_is_finished(upgrade_list: &[&dyn Upgrade], group_i: usize) -> bool {
    let mut any = false;
    let all_maxed = upgrade_list
        .iter()
        .filter(|u| u.group() == group_i)
        .all(|u| {
            any = true;
            (0..u.count_tracks()).all(|t| u.track_next_cost(t).is_none())
        });
    any && all_maxed
}

fn build_tree_items(upgrades: &Upgrades) -> Vec<TreeItem<'static, usize>> {
    let upgrade_list = upgrades.upgrades();
    let group_unlocks = groups_are_unlocked(upgrades);
    let groups = (0..group_unlocks.len())
        .filter(|i| group_unlocks[*i])
        .map(|group_i| {
            let label = if group_is_finished(&upgrade_list, group_i) {
                format!("Level {group_i} upgrades {GROUP_FINISHED_ICON}")
            } else {
                format!("Level {group_i} upgrades")
            };
            TreeItem::new(group_i, label, render_group_items(&upgrade_list, group_i))
        });
    groups.map(|item| item.unwrap()).collect()
}

pub(super) fn open_all_upgrade_nodes(state: &mut TreeState<usize>, upgrades: &Upgrades) {
    let upgrade_list = upgrades.upgrades();
    let group_unlocks = groups_are_unlocked(upgrades);
    for group_i in 0..group_unlocks.len() {
        if !group_unlocks[group_i] {
            continue;
        }
        let count = upgrade_list.iter().filter(|u| u.group() == group_i).count();
        for upgrade_i in 0..count {
            state.open(vec![group_i, upgrade_i]);
        }
    }
}

pub(super) fn create_tree_widget(upgrades: &Upgrades) -> TreeWidget<'static> {
    let mut widget = TreeWidget::new(
        build_tree_items(upgrades),
        |tree_items| {
            Tree::new(tree_items)
                .unwrap()
                .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
                .experimental_scrollbar(Some(Scrollbar::default()))
        },
        TreeState::default(),
    );
    widget.with_tree_state_mut(|state| open_all_upgrade_nodes(state, upgrades));
    widget
}

impl Widget for &mut TreeWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.with_mut(|tree| {
            StatefulWidget::render(&*tree.tree, area, buf, tree.tree_state);
        });
    }
}

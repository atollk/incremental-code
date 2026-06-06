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
        .next_level()
        .map(|u| u.value_text().to_string())
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

fn cost_style(cost: Option<Resources>) -> Style {
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
}

fn render_track_items(upgrade: &dyn Upgrade) -> Vec<TreeItem<'static, usize>> {
    (0..upgrade.count_tracks())
        .filter(|&track| upgrade.track_next_cost(track).is_some())
        .map(|track| {
            let track_level = upgrade.track_get_level(track);
            let max_level = upgrade.max_level();
            let cost = upgrade.track_next_cost(track);
            let cost_str = match &cost {
                None => "maxed".to_string(),
                Some(c) => c.fmt_oneline().to_string(),
            };
            let style = cost_style(cost);
            let line = Line::from(vec![
                Span::raw(format!("-  {:>3}/{:<3}  ", track_level, max_level)),
                Span::styled(cost_str, style),
            ]);
            TreeItem::new_leaf(track, line)
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

fn build_tree_items(upgrades: &Upgrades) -> Vec<TreeItem<'static, usize>> {
    let upgrade_list = upgrades.upgrades();
    let group_unlocks = groups_are_unlocked(upgrades);
    let groups = (0..group_unlocks.len())
        .filter(|i| group_unlocks[*i])
        .map(|group_i| {
            TreeItem::new(
                group_i,
                format!("Level {group_i} upgrades"),
                render_group_items(&upgrade_list, group_i),
            )
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

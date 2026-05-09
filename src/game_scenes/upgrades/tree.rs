use crate::game_state::{Upgrade, Upgrades, with_game_state};
use crate::widgets::tree::{Tree, TreeItem, TreeState};
use itertools::Itertools;
use logos::Source;
use ouroboros::self_referencing;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::text::{Line, Span};
use ratatui_core::widgets::{StatefulWidget, Widget};
use std::cmp::max;
use std::hash::Hash;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[self_referencing]
pub(super) struct TreeWidget<'a> {
    pub tree_items: Vec<TreeItem<'a, usize>>,
    #[borrows(tree_items)]
    #[covariant]
    pub tree: Tree<'this, usize>,
    pub tree_state: TreeState<usize>,
}

pub fn find_item_in_tree<'a, 'b, T: Eq + Hash + Clone>(
    tree_items: &'b [TreeItem<'a, T>],
    identifier_path: &[T],
) -> Option<&'b TreeItem<'a, T>> {
    if let [head, tail @ ..] = identifier_path {
        tree_items
            .iter()
            .find(|child| child.identifier() == head)
            .and_then(|child| child.find_child(tail))
    } else {
        None
    }
}

struct TreeColumns<T> {
    name: T,
    level: T,
    level_up_cost: T,
    values: T,
}

fn render_column_texts(upgrade: &dyn Upgrade) -> TreeColumns<String> {
    let level_str = upgrade.format_level_str();
    let cost_str = upgrade.format_cost_str();
    let current_value_str = upgrade.value_text();
    let next_value_str = upgrade
        .next_level()
        .map(|u| u.value_text().to_string())
        .unwrap_or_default();

    TreeColumns {
        name: upgrade.name().to_string(),
        level: level_str,
        level_up_cost: cost_str,
        values: match (current_value_str, next_value_str.as_str()) {
            ("", "") => "".to_string(),
            ("", _) => format!("-> {}", next_value_str),
            (_, "") => current_value_str.to_string(),
            (_, _) => format!("{} -> {}", current_value_str, next_value_str),
        },
    }
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
            let level_width = column_sizes.level;
            let level_up_cost_width = column_sizes.level_up_cost;
            let values_width = column_sizes.values;
            let cost_style = {
                let cost = u.next_level_cost();
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
            };
            let spans = vec![
                Span::raw(format!("{:<name_width$}", tc.name)),
                Span::raw("     "),
                Span::raw(format!("{:<level_width$}", tc.level)),
                Span::raw("     "),
                Span::styled(
                    format!("{:<level_up_cost_width$}", tc.level_up_cost),
                    cost_style,
                ),
                Span::raw("    "),
                Span::raw(format!("{:^values_width$}", tc.values)),
            ];
            TreeItem::new_leaf(i, Line::from_iter(spans))
        })
        .collect()
}

fn build_tree_items(upgrades: &Upgrades) -> Vec<TreeItem<'static, usize>> {
    let upgrade_list = upgrades.upgrades();
    let group_unlocks = [
        true,
        upgrades.unlock_level1.value(),
        upgrades.unlock_level2.value(),
        upgrades.unlock_level3.value(),
        upgrades.unlock_level4.value(),
        upgrades.unlock_level5.value(),
        upgrades.unlock_level6.value(),
    ];
    let groups = (0..=6).filter(|i| group_unlocks[*i]).map(|group_i| {
        TreeItem::new(
            group_i,
            format!("Level {group_i} upgrades"),
            render_group_items(&upgrade_list, group_i),
        )
    });
    groups.map(|item| item.unwrap()).collect()
}

pub(super) fn create_tree_widget(upgrades: &Upgrades) -> TreeWidget<'static> {
    TreeWidget::new(
        build_tree_items(&upgrades),
        |tree_items| {
            Tree::new(tree_items)
                .unwrap()
                .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        },
        TreeState::default(),
    )
}

impl Widget for &mut TreeWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.with_mut(|tree| {
            StatefulWidget::render(&*tree.tree, area, buf, tree.tree_state);
        });
    }
}

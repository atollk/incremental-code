use crate::game_state::{Upgrade, Upgrades, with_game_state};
use crate::widgets::tree::{Tree, TreeItem, TreeState};
use ouroboros::self_referencing;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::text::{Line, Span};
use ratatui_core::widgets::{StatefulWidget, Widget};
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

fn render_upgrade(upgrade: &dyn Upgrade, name_width: usize, level_width: usize) -> Line<'static> {
    const EMPTY_BOX: char = '🔲';
    const FULL_BOX: char = '⬛';
    let level_str = upgrade.format_level_str(EMPTY_BOX, FULL_BOX);
    let cost_str = upgrade.format_cost_str();
    let current_value_str = upgrade.value_text();
    let next_value_str = upgrade
        .next_level()
        .map(|u| u.value_text().to_string())
        .unwrap_or_default();
    Line::from_iter(vec![
        Span::raw(format!("{:<name_width$}", upgrade.name())),
        Span::raw("     "),
        Span::raw(format!(
            "{}{}",
            level_str,
            " ".repeat((level_width * FULL_BOX.width().unwrap()).saturating_sub(level_str.width()))
        )),
        Span::raw("     "),
        Span::raw(format!("{} -> {}", current_value_str, next_value_str)),
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

fn build_tree_items(upgrades: &Upgrades) -> Vec<TreeItem<'static, usize>> {
    let upgrade_list = upgrades.upgrades();
    let name_width = upgrade_list
        .iter()
        .map(|u| u.name().len())
        .max()
        .unwrap_or(0);
    let level_width = upgrade_list
        .iter()
        .map(|u| u.max_level())
        .max()
        .unwrap_or(0);
    let group_unlocks = [
        true,
        upgrades.unlock_level1.value(),
        upgrades.unlock_level2.value(),
        upgrades.unlock_level3.value(),
        upgrades.unlock_level4.value(),
        upgrades.unlock_level5.value(),
        upgrades.unlock_level6.value(),
    ];
    let groups = (0..=6).filter(|i| group_unlocks[*i]).map(|group| {
        TreeItem::new(
            group,
            format!("Level {group} upgrades"),
            upgrade_list
                .iter()
                .enumerate()
                .filter(|(_, u)| u.group() == group)
                .map(|(i, &u)| {
                    TreeItem::new_leaf(i, render_upgrade(u, name_width, level_width as usize))
                })
                .collect(),
        )
    });
    groups.map(|item| item.unwrap()).collect()
}

pub(super) fn create_tree_widget(upgrades: &Upgrades) -> TreeWidget<'static> {
    let mut widget = TreeWidget::new(
        build_tree_items(&upgrades),
        |tree_items| {
            Tree::new(tree_items)
                .unwrap()
                .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        },
        TreeState::default(),
    );
    widget
}

impl Widget for &mut TreeWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.with_mut(|tree| {
            StatefulWidget::render(&*tree.tree, area, buf, tree.tree_state);
        });
    }
}

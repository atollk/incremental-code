use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind, MouseEventKind};
use crate::widgets::tree::TreeState;
use ratatui_core::layout::Position;
use std::hash::Hash;

impl<Identifier: Clone + Hash + PartialEq + Eq> TreeState<Identifier> {
    pub fn process_input_event(&mut self, event: &Event) {
        match event {
            Event::KeyEvent(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            self.toggle_selected();
                        }
                        KeyCode::Left => {
                            self.key_left();
                        }
                        KeyCode::Right => {
                            self.key_right();
                        }
                        KeyCode::Down => {
                            self.key_down();
                        }
                        KeyCode::Up => {
                            self.key_up();
                        }
                        KeyCode::Esc => {
                            self.select(Vec::new());
                        }
                        KeyCode::Home => {
                            self.select_first();
                        }
                        KeyCode::End => {
                            self.select_last();
                        }
                        KeyCode::PageDown => {
                            self.scroll_down(3);
                        }
                        KeyCode::PageUp => {
                            self.scroll_up(3);
                        }
                        _ => (),
                    }
                }
            }
            Event::MouseEvent(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => {
                    self.scroll_down(1);
                }
                MouseEventKind::ScrollUp => {
                    self.scroll_up(1);
                }
                MouseEventKind::Down(_button) => {
                    self.click_at(Position::new(mouse.column, mouse.row));
                }
                _ => (),
            },
        };
    }
}

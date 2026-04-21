use crate::backend::input::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use crate::widgets::code_editor::actions::DefaultAction;
use crate::widgets::code_editor::editor::Editor;
use ratatui_core::layout::Rect;

/// Signals returned to the caller after processing a key event.
pub enum EditorCommand {
    /// The event was consumed and handled by the editor.
    Handled,
    /// The user requested to save and exit (Ctrl+S).
    SaveAndExit,
    /// The user requested to exit (Esc).
    Exit,
}

/// Translates a key event into an editor action and applies it.
///
/// Returns an `EditorCommand` so the caller can handle scene-level transitions
/// (saving, dialogs, etc.) without the editor needing to know about them.
pub fn apply_key_event(editor: &mut Editor, key: &KeyEvent) -> Option<EditorCommand> {
    if key.kind == KeyEventKind::Release {
        return None;
    }
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    if ctrl {
        match key.code {
            KeyCode::Char('s') | KeyCode::Char('S') => {
                return Some(EditorCommand::SaveAndExit);
            }
            KeyCode::Char('z') | KeyCode::Char('Z') => {
                if shift {
                    editor.apply(DefaultAction::Redo);
                } else {
                    editor.apply(DefaultAction::Undo);
                }
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => editor.apply(DefaultAction::Redo),
            KeyCode::Char('a') | KeyCode::Char('A') => editor.apply(DefaultAction::SelectAll),
            KeyCode::Char('c') | KeyCode::Char('C') => editor.apply(DefaultAction::Copy),
            KeyCode::Char('x') | KeyCode::Char('X') => editor.apply(DefaultAction::Cut),
            KeyCode::Char('v') | KeyCode::Char('V') => editor.apply(DefaultAction::Paste),
            KeyCode::Char('/') => editor.apply(DefaultAction::ToggleComment),
            KeyCode::Char('d') | KeyCode::Char('D') => editor.apply(DefaultAction::Duplicate),
            _ => return None,
        }
    } else {
        match key.code {
            KeyCode::Esc => return Some(EditorCommand::Exit),
            KeyCode::Left => editor.apply(DefaultAction::MoveLeft { shift }),
            KeyCode::Right => editor.apply(DefaultAction::MoveRight { shift }),
            KeyCode::Up => editor.apply(DefaultAction::MoveUp { shift }),
            KeyCode::Down => editor.apply(DefaultAction::MoveDown { shift }),
            KeyCode::Backspace => editor.apply(DefaultAction::Delete),
            KeyCode::Delete => {
                editor.apply(DefaultAction::MoveRight { shift: false });
                editor.apply(DefaultAction::Delete);
            }
            KeyCode::Enter => editor.apply(DefaultAction::InsertNewline),
            KeyCode::Tab => editor.apply(DefaultAction::Indent),
            KeyCode::BackTab => editor.apply(DefaultAction::UnIndent),
            KeyCode::Char(c) => editor.apply(DefaultAction::InsertText { text: c.to_string() }),
            _ => return None,
        }
    }

    Some(EditorCommand::Handled)
}

/// Translates a mouse event into editor operations and applies them.
pub fn apply_mouse_event(editor: &mut Editor, mouse: &MouseEvent, area: &Rect) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(cursor) = editor.cursor_from_mouse(mouse.column, mouse.row, area) {
                editor.handle_mouse_down(cursor);
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if let Some(cursor) = editor.cursor_from_mouse(mouse.column, mouse.row, area) {
                editor.handle_mouse_drag(cursor);
            }
        }
        MouseEventKind::ScrollUp => editor.scroll_up(),
        MouseEventKind::ScrollDown => editor.scroll_down(area.height as usize),
        _ => {}
    }
}

use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use crate::basic_terminal_app::App;
use crate::blinking_cursor::BlinkingCursor;
use ratatui_code_editor::actions::DefaultAction;
use ratatui_code_editor::editor::Editor;
use ratatui_code_editor::theme::vesper;
use ratatui_core::layout::{Position, Rect};
use std::sync::OnceLock;

pub struct CodeEditorDemo {
    editor: Editor,
}

fn epoch() -> web_time::Instant {
    static EPOCH: OnceLock<web_time::Instant> = OnceLock::new();
    *EPOCH.get_or_init(web_time::Instant::now)
}

fn key_to_action(key: &KeyEvent) -> Option<DefaultAction> {
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let _alt = key.modifiers.contains(KeyModifiers::ALT);

    match key.code {
        KeyCode::Char('÷') => Some(DefaultAction::ToggleComment),
        KeyCode::Char('z') if ctrl => Some(DefaultAction::Undo),
        KeyCode::Char('y') if ctrl => Some(DefaultAction::Redo),
        KeyCode::Char('c') if ctrl => Some(DefaultAction::Copy),
        KeyCode::Char('v') if ctrl => Some(DefaultAction::Paste),
        KeyCode::Char('x') if ctrl => Some(DefaultAction::Cut),
        KeyCode::Char('k') if ctrl => Some(DefaultAction::DeleteLine),
        KeyCode::Char('d') if ctrl => Some(DefaultAction::Duplicate),
        KeyCode::Char('a') if ctrl => Some(DefaultAction::SelectAll),
        KeyCode::Left => Some(DefaultAction::MoveLeft { shift }),
        KeyCode::Right => Some(DefaultAction::MoveRight { shift }),
        KeyCode::Up => Some(DefaultAction::MoveUp { shift }),
        KeyCode::Down => Some(DefaultAction::MoveDown { shift }),
        KeyCode::Backspace => Some(DefaultAction::Delete),
        KeyCode::Enter => Some(DefaultAction::InsertNewline),
        KeyCode::Char(c) => Some(DefaultAction::InsertText {
            text: c.to_string(),
        }),
        KeyCode::Tab => Some(DefaultAction::Indent),
        KeyCode::BackTab => Some(DefaultAction::UnIndent),
        _ => None,
    }
}

impl CodeEditorDemo {
    pub fn input(&mut self, key: &KeyEvent, area: &Rect) -> anyhow::Result<()> {
        if let Some(action) = key_to_action(key) {
            self.editor.apply(action);
        }
        self.editor.focus(&area);
        Ok(())
    }
}

impl App for CodeEditorDemo {
    fn frame(
        &mut self,
        events: &[Event],
        frame: &mut ratatui_core::terminal::Frame,
    ) -> anyhow::Result<bool> {
        for event in events {
            match event {
                Event::KeyEvent(key) => match key.code {
                    KeyCode::Esc => {
                        return Ok(true);
                    }
                    _ => {
                        if key.kind == KeyEventKind::Press {
                            self.input(key, &frame.area())?;
                        }
                    }
                },
                Event::MouseEvent(_) => {}
            }
        }

        frame.render_widget(&self.editor, frame.area());
        let cursor = self.editor.get_visible_cursor(&frame.area());
        if let Some((x, y)) = cursor {
            frame.set_cursor_position(Position::new(x, y));
            frame.render_widget(BlinkingCursor::new(x, y), frame.area());
        }

        Ok(false)
    }
}

impl Default for CodeEditorDemo {
    fn default() -> Self {
        let code = "def foo():\n  pass";
        let lang = Some(tree_sitter_python::LANGUAGE.into());
        let editor = Editor::new_with_highlights(
            lang,
            code,
            vesper(),
            Some(tree_sitter_python::HIGHLIGHTS_QUERY.to_string()),
        )
        .unwrap();
        CodeEditorDemo { editor }
    }
}

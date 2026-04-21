use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crate::basic_terminal_app::App;
use crate::widgets::blinking_cursor::BlinkingCursor;
use crate::widgets::code_editor;
use crate::widgets::code_editor::actions::DefaultAction;
use crate::widgets::code_editor::editor::Editor;
use crate::widgets::code_editor::python_logos::PythonLangToken;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Style};
use std::collections::HashMap;

pub struct CodeEditorDemo {
    editor: Editor,
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
        self.editor.focus(area);
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
        let lang = code_editor::python_logos::python_language(prism_theme());
        let editor = Editor::new(Box::new(lang), code);
        CodeEditorDemo { editor }
    }
}

fn prism_theme() -> HashMap<PythonLangToken, Style> {
    let colors = [
        // Identifiers
        (PythonLangToken::Identifier, "#82aaff"),
        // Literals
        (PythonLangToken::Int, "#f78c6c"),
        (PythonLangToken::Float, "#f78c6c"),
        (PythonLangToken::Imaginary, "#f78c6c"),
        // Strings
        (PythonLangToken::StringLiteral, "#c3e88d"),
        (PythonLangToken::RawStringLiteral, "#c3e88d"),
        (PythonLangToken::TripleStringLiteral, "#c3e88d"),
        (PythonLangToken::RawTripleStringLiteral, "#c3e88d"),
        (PythonLangToken::FStringStart, "#c3e88d"),
        // Comments
        (PythonLangToken::Comment, "#546e7a"),
        // Constants (True / False / None)
        (PythonLangToken::KwTrue, "#ff9cac"),
        (PythonLangToken::KwFalse, "#ff9cac"),
        (PythonLangToken::KwNone, "#ff9cac"),
        // Keywords
        (PythonLangToken::KwAnd, "#c792ea"),
        (PythonLangToken::KwAs, "#c792ea"),
        (PythonLangToken::KwAssert, "#c792ea"),
        (PythonLangToken::KwAsync, "#c792ea"),
        (PythonLangToken::KwAwait, "#c792ea"),
        (PythonLangToken::KwBreak, "#c792ea"),
        (PythonLangToken::KwClass, "#c792ea"),
        (PythonLangToken::KwContinue, "#c792ea"),
        (PythonLangToken::KwDef, "#c792ea"),
        (PythonLangToken::KwDel, "#c792ea"),
        (PythonLangToken::KwElif, "#c792ea"),
        (PythonLangToken::KwElse, "#c792ea"),
        (PythonLangToken::KwExcept, "#c792ea"),
        (PythonLangToken::KwFinally, "#c792ea"),
        (PythonLangToken::KwFor, "#c792ea"),
        (PythonLangToken::KwFrom, "#c792ea"),
        (PythonLangToken::KwGlobal, "#c792ea"),
        (PythonLangToken::KwIf, "#c792ea"),
        (PythonLangToken::KwImport, "#c792ea"),
        (PythonLangToken::KwIn, "#c792ea"),
        (PythonLangToken::KwIs, "#c792ea"),
        (PythonLangToken::KwLambda, "#c792ea"),
        (PythonLangToken::KwMatch, "#c792ea"),
        (PythonLangToken::KwCase, "#c792ea"),
        (PythonLangToken::KwType, "#c792ea"),
        (PythonLangToken::KwNonlocal, "#c792ea"),
        (PythonLangToken::KwNot, "#c792ea"),
        (PythonLangToken::KwOr, "#c792ea"),
        (PythonLangToken::KwPass, "#c792ea"),
        (PythonLangToken::KwRaise, "#c792ea"),
        (PythonLangToken::KwReturn, "#c792ea"),
        (PythonLangToken::KwTry, "#c792ea"),
        (PythonLangToken::KwWhile, "#c792ea"),
        (PythonLangToken::KwWith, "#c792ea"),
        (PythonLangToken::KwYield, "#c792ea"),
        // Operators
        (PythonLangToken::Plus, "#89ddff"),
        (PythonLangToken::Minus, "#89ddff"),
        (PythonLangToken::Star, "#89ddff"),
        (PythonLangToken::Slash, "#89ddff"),
        (PythonLangToken::Percent, "#89ddff"),
        (PythonLangToken::DoubleStar, "#89ddff"),
        (PythonLangToken::SlashSlash, "#89ddff"),
        (PythonLangToken::At, "#89ddff"),
        (PythonLangToken::Tilde, "#89ddff"),
        (PythonLangToken::Ampersand, "#89ddff"),
        (PythonLangToken::Pipe, "#89ddff"),
        (PythonLangToken::Caret, "#89ddff"),
        (PythonLangToken::LessLess, "#89ddff"),
        (PythonLangToken::GreaterGreater, "#89ddff"),
        (PythonLangToken::Less, "#89ddff"),
        (PythonLangToken::Greater, "#89ddff"),
        (PythonLangToken::LessEqual, "#89ddff"),
        (PythonLangToken::GreaterEqual, "#89ddff"),
        (PythonLangToken::EqEqual, "#89ddff"),
        (PythonLangToken::NotEqual, "#89ddff"),
        (PythonLangToken::Equal, "#89ddff"),
        (PythonLangToken::Arrow, "#89ddff"),
        (PythonLangToken::ColonEqual, "#89ddff"),
        (PythonLangToken::PlusEqual, "#89ddff"),
        (PythonLangToken::MinusEqual, "#89ddff"),
        (PythonLangToken::StarEqual, "#89ddff"),
        (PythonLangToken::SlashEqual, "#89ddff"),
        (PythonLangToken::PercentEqual, "#89ddff"),
        (PythonLangToken::AmpersandEqual, "#89ddff"),
        (PythonLangToken::PipeEqual, "#89ddff"),
        (PythonLangToken::CaretEqual, "#89ddff"),
        (PythonLangToken::AtEqual, "#89ddff"),
        (PythonLangToken::DoubleStarEqual, "#89ddff"),
        (PythonLangToken::SlashSlashEqual, "#89ddff"),
        (PythonLangToken::LessLessEqual, "#89ddff"),
        (PythonLangToken::GreaterGreaterEqual, "#89ddff"),
    ];
    colors
        .into_iter()
        .map(|(token, hex)| {
            let (r, g, b) = code_editor::utils::rgb(hex);
            (token, Style::default().fg(Color::Rgb(r, g, b)))
        })
        .collect()
}

use crate::backend::events::Event;
use crate::game_scenes::base::{Scene, SceneSwitch};
use crate::game_scenes::home_terminal::HomeTerminalScene;
use crate::game_state::with_game_state;
use crate::widgets::code_editor::editor::Editor;
use crate::widgets::code_editor::input::{apply_key_event, apply_mouse_event, EditorCommand};
use crate::widgets::code_editor::python_logos;
use crate::widgets::dialog::{ConfirmDialog, ConfirmResult};
use ratatui_core::terminal::Frame;
use std::collections::HashMap;
use web_time::Duration;

pub struct CodeEditorScene {
    editor: Editor,
    original_code: String,
    confirm_dialog: Option<ConfirmDialog>,
}

impl CodeEditorScene {
    pub fn new() -> Self {
        let code = with_game_state(|state| state.program_code.clone());
        let lang = Box::new(python_logos::python_language(HashMap::new()));
        let editor = Editor::new(lang, &code);
        CodeEditorScene {
            editor,
            original_code: code,
            confirm_dialog: None,
        }
    }

    fn save_code(&self) {
        let content = self.editor.get_content();
        with_game_state(|state| {
            state.program_code = content.clone();
        });
    }
}

impl Scene for CodeEditorScene {
    fn frame(&mut self, events: &[Event], frame: &mut Frame, _time_delta: Duration) -> SceneSwitch {
        let area = frame.area();

        // ConfirmingExit mode: dialog is active
        if self.confirm_dialog.is_some() {
            for event in events {
                self.confirm_dialog.as_mut().unwrap().handle_event(event);
            }
            let result = self.confirm_dialog.as_ref().unwrap().result();
            let switch = match result {
                Some(ConfirmResult::Yes) => {
                    self.save_code();
                    SceneSwitch::SwitchTo(Box::new(HomeTerminalScene::new()))
                }
                Some(ConfirmResult::No) => SceneSwitch::SwitchTo(Box::new(HomeTerminalScene::new())),
                Some(ConfirmResult::Cancel) => {
                    self.confirm_dialog = None;
                    SceneSwitch::NoSwitch
                }
                None => SceneSwitch::NoSwitch,
            };
            frame.render_widget(&self.editor, area);
            if let Some(dialog) = &self.confirm_dialog {
                frame.render_widget(dialog, area);
            }
            return switch;
        }

        // Editing mode
        let mut switch = SceneSwitch::NoSwitch;
        'events: for event in events {
            match event {
                Event::KeyEvent(key) => {
                    match apply_key_event(&mut self.editor, key) {
                        Some(EditorCommand::SaveAndExit) => {
                            self.save_code();
                            switch = SceneSwitch::SwitchTo(Box::new(HomeTerminalScene::new()));
                            break 'events;
                        }
                        Some(EditorCommand::Exit) => {
                            if self.editor.get_content() == self.original_code {
                                switch = SceneSwitch::SwitchTo(Box::new(HomeTerminalScene::new()));
                            } else {
                                self.confirm_dialog = Some(ConfirmDialog::new(
                                    "Unsaved Changes",
                                    "Save changes? [Y]es  [N]o  [Esc] Cancel",
                                ));
                            }
                            break 'events;
                        }
                        Some(EditorCommand::Handled) | None => {}
                    }
                }
                Event::MouseEvent(mouse) => {
                    apply_mouse_event(&mut self.editor, mouse, &area);
                }
            }
        }

        self.editor.focus(&area);
        frame.render_widget(&self.editor, area);
        switch
    }
}

use crate::backend::events::Event;
use crate::game_scenes::base::{Scene, SceneSwitch};
use crate::game_scenes::home_terminal::HomeTerminalScene;
use crate::game_state::{with_game_state, with_game_state_mut};
use crate::widgets::code_editor::editor::Editor;
use crate::widgets::code_editor::input::{EditorCommand, apply_key_event, apply_mouse_event};
use crate::widgets::code_editor::not_python_logos::{
    not_python_default_theme, not_python_language,
};
use crate::widgets::dialog::{ConfirmDialog, ConfirmResult};
use ratatui_core::terminal::Frame;
use web_time::Duration;

pub struct CodeEditorScene {
    editor: Editor,
    original_code: String,
    confirm_dialog: Option<ConfirmDialog>,
}

impl Default for CodeEditorScene {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeEditorScene {
    pub fn new() -> Self {
        let code = with_game_state(|state| state.program_code.clone());
        let lang = not_python_language(not_python_default_theme());
        let editor = Editor::new(Box::new(lang), &code);
        CodeEditorScene {
            editor,
            original_code: code,
            confirm_dialog: None,
        }
    }

    fn save_code(&self) {
        let content = self.editor.get_content();
        with_game_state_mut(|state| {
            state.program_code = content.clone();
        });
    }
}

impl Scene for CodeEditorScene {
    fn frame(&mut self, events: &[Event], frame: &mut Frame, _time_delta: Duration) -> SceneSwitch {
        // ConfirmingExit mode: dialog is active
        if self.confirm_dialog.is_some() {
            for event in events {
                self.confirm_dialog.as_mut().unwrap().handle_event(event);
            }
            let result = self.confirm_dialog.as_ref().unwrap().result();
            let dialog_scene_switch = match result {
                Some(ConfirmResult::Yes) => {
                    self.save_code();
                    SceneSwitch::SwitchTo(Box::new(HomeTerminalScene::new()))
                }
                Some(ConfirmResult::No) => {
                    SceneSwitch::SwitchTo(Box::new(HomeTerminalScene::new()))
                }
                Some(ConfirmResult::Cancel) => {
                    self.confirm_dialog = None;
                    SceneSwitch::NoSwitch
                }
                None => SceneSwitch::NoSwitch,
            };
            frame.render_widget(&self.editor, frame.area());
            if let Some(dialog) = &self.confirm_dialog {
                frame.render_widget(dialog, frame.area());
            }
            dialog_scene_switch?;
        }

        // Editing mode
        let mut switch = SceneSwitch::NoSwitch;
        'events: for event in events {
            match event {
                Event::KeyEvent(key) => match apply_key_event(&mut self.editor, key) {
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
                },
                Event::MouseEvent(mouse) => {
                    apply_mouse_event(&mut self.editor, mouse, &frame.area());
                }
            }
        }
        self.editor.focus(&frame.area());
        frame.render_widget(&self.editor, frame.area());
        switch
    }
}

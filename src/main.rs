mod backend;

use crate::backend::backend::{BackendSuite, TerminalApp};
use ratatui::widgets::{Block, Paragraph};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Stylize;
use ratatui_core::symbols::border;
use ratatui_core::text::{Line, Text};
use ratatui_core::widgets::Widget;

pub fn main() {
    let tapp = TApp {
        terminal: None,
        app_state: AppState {
            counter: 0,
            exit: false,
        },
    };
    backend::BACKEND_INSTANCE
        .lock()
        .unwrap()
        .run(tapp)
        .unwrap();
}

struct AppState {
    counter: i32,
    exit: bool,
}

struct TApp {
    terminal: Option<ratatui::Terminal<backend::BackendType>>,
    app_state: AppState,
}

impl TerminalApp<backend::BackendType> for TApp {
    fn init(&mut self, backend: backend::BackendType) -> anyhow::Result<()> {
        let terminal = ratatui::Terminal::new(backend).unwrap();
        self.terminal = Some(terminal);
        Ok(())
    }

    fn frame(&mut self) -> anyhow::Result<()> {
        let terminal = self.terminal.as_mut().unwrap();
        let app_state = &self.app_state;
        terminal.draw(|frame: &mut ratatui::Frame| frame.render_widget(app_state, frame.area()))?;
        if self.app_state.exit {
            return Err(anyhow::anyhow!("exit"));
        }
        Ok(())
    }

    fn backend(&self) -> &backend::BackendType {
        self.terminal.as_ref().unwrap().backend()
    }

    fn backend_mut(&mut self) -> &mut backend::BackendType {
        self.terminal.as_mut().unwrap().backend_mut()
    }
}

impl Widget for &AppState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

use crate::backend::backend::BackendSuite;
use ratatui::DefaultTerminal;
use ratatui::widgets::{Block, Paragraph};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Stylize;
use ratatui_core::symbols::border;
use ratatui_core::terminal::Frame;
use ratatui_core::text::{Line, Text};
use ratatui_core::widgets::Widget;
use std::io;

pub fn main() {
    crate::backend::BACKEND_INSTANCE.lock().unwrap().run(|be| {
        let mut terminal = ratatui::Terminal::new(be).unwrap();
        let mut app = App {
            counter: 0,
            exit: false,
        };
        app.run(&mut terminal).unwrap();
    }).unwrap();
}

struct App {
    counter: i32,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            // self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }
}

impl Widget for &App {
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

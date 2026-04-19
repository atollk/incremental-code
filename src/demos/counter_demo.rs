use crate::backend::events::Event;
use crate::backend::input::KeyCode;
use crate::basic_terminal_app::App;
use ratatui::widgets::{Block, Paragraph};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Stylize;
use ratatui_core::symbols::border;
use ratatui_core::text::{Line, Text};
use ratatui_core::widgets::Widget;

#[derive(Default)]
pub struct CounterDemo {
    counter: i32,
}

impl App for CounterDemo {
    fn frame(
        &mut self,
        events: &[Event],
        frame: &mut ratatui_core::terminal::Frame,
    ) -> anyhow::Result<bool> {
        for event in events {
            match event {
                Event::KeyEvent(key) => match key.code {
                    KeyCode::Left => {
                        self.counter -= 1;
                    }
                    KeyCode::Right => {
                        self.counter += 1;
                    }
                    KeyCode::Char('q') => {
                        return Ok(true);
                    }
                    _ => {}
                },
                Event::MouseEvent(_) => {}
            }
        }
        frame.render_widget(&*self, frame.area());
        Ok(false)
    }
}


impl Widget for &CounterDemo {
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

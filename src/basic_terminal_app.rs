use crate::backend;
use crate::backend::backend::BackendSuite;
use crate::backend::backend::TerminalApp;
use crate::backend::events::Event;

pub trait App {
    fn frame(&mut self, events: &[Event], frame: &mut ratatui::Frame) -> anyhow::Result<bool>;
}

pub struct BasicTerminalApp<A: App> {
    terminal: Option<ratatui::Terminal<backend::BackendType>>,
    app: A,
}

impl<A: App + 'static> BasicTerminalApp<A> {
    pub(crate) fn new(app: A) -> Self {
        BasicTerminalApp {
            terminal: None,
            app,
        }
    }

    pub(crate) fn run(self) -> anyhow::Result<()> {
        backend::BACKEND_INSTANCE.lock().unwrap().run(self)
    }
}

impl<A: App> TerminalApp<backend::BackendType> for BasicTerminalApp<A> {
    fn init(&mut self, backend: backend::BackendType) -> anyhow::Result<()> {
        let terminal = ratatui::Terminal::new(backend).unwrap();
        self.terminal = Some(terminal);
        Ok(())
    }

    fn frame(&mut self, events: &[Event]) -> anyhow::Result<bool> {
        let terminal = self.terminal.as_mut().unwrap();
        let app = &mut self.app;
        let mut exit = false;
        terminal.draw(|frame: &mut ratatui::Frame| exit = app.frame(events, frame).unwrap())?;
        Ok(exit)
    }

    fn backend(&self) -> &backend::BackendType {
        self.terminal.as_ref().unwrap().backend()
    }

    fn backend_mut(&mut self) -> &mut backend::BackendType {
        self.terminal.as_mut().unwrap().backend_mut()
    }
}

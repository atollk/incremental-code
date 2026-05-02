use crate::backend;
use crate::backend::backend::TerminalApp;
use crate::backend::events::Event;

/// High-level application trait driven by [`BasicTerminalApp`].
pub trait App {
    /// Called once per frame with all events collected since the last frame.
    ///
    /// Returns `true` to request a clean exit of the application loop.
    fn frame(&mut self, events: &[Event], frame: &mut ratatui::Frame) -> anyhow::Result<bool>;
}

/// Wraps an [`App`] and wires it to the platform-specific [`BackendSuite`](crate::backend::backend::BackendSuite).
pub struct BasicTerminalApp<A: App> {
    terminal: Option<ratatui::Terminal<backend::BackendType>>,
    app: A,
}

impl<A: App + 'static> BasicTerminalApp<A> {
    /// Wraps `app` in a `BasicTerminalApp` ready to be passed to the backend.
    pub(crate) fn new(app: A) -> Self {
        BasicTerminalApp {
            terminal: None,
            app,
        }
    }

    /// Hand the app to the global backend and block until the app requests an exit.
    pub(crate) fn run(&mut self) -> anyhow::Result<()> {
        backend::with_backend(|backend| backend.run(self))
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

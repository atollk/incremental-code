use crate::backend::input::{KeyEvent, MouseEvent};

/// A normalized input event produced by the active backend.
#[derive(Debug)]
pub enum Event {
    KeyEvent(KeyEvent),
    MouseEvent(MouseEvent),
    MetaEvent(MetaEvent),
}

pub(crate) trait IntoEvent {
    /// Converts a backend-specific raw event into a normalised [`Event`], returning `None` for events that are not handled.
    fn into_event(self) -> Option<Event>;
}

#[derive(Debug)]
pub enum MetaEvent {
    ResizeApp,
    SigTerm,
}

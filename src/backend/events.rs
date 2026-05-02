use crate::backend::input::{KeyEvent, MouseEvent};

/// A normalised input event produced by the active backend.
pub enum Event {
    KeyEvent(KeyEvent),
    MouseEvent(MouseEvent),
}

pub(crate) trait IntoEvent {
    /// Converts a backend-specific raw event into a normalised [`Event`], returning `None` for events that are not handled.
    fn into_event(self) -> Option<Event>;
}

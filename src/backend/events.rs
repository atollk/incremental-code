use crate::backend::input::{KeyEvent, MouseEvent};

pub(crate) enum Event {
    KeyEvent(KeyEvent),
    MouseEvent(MouseEvent),
}

pub(crate) trait IntoEvent {
    fn into_event(self) -> Option<Event>;
}
use crate::backend::input::{KeyEvent, MouseEvent};

pub enum Event {
    KeyEvent(KeyEvent),
    MouseEvent(MouseEvent),
}

pub(crate) trait IntoEvent {
    fn into_event(self) -> Option<Event>;
}
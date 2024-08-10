pub(crate) mod input;
mod key;
#[cfg(feature = "mouse-support")]
pub(crate) mod mouse;

pub use key::{KeyEvent, KeyEventHandler, KeyEventRegister};
pub use mouse::{MouseEvent, MouseEventHandler};

use crate::EditorState;

/// Handles key and mouse events.
#[derive(Default)]
pub struct EditorEvent {
    pub key_handler: KeyEventHandler,
}

impl EditorEvent {
    /// Creates a new `EditorEvent` handler.
    #[must_use]
    pub fn new(key_handler: KeyEventHandler) -> Self {
        Self { key_handler }
    }

    pub fn on_key_event<T>(&mut self, event: T, state: &mut EditorState)
    where
        T: Into<KeyEvent>,
    {
        self.key_handler.on_event(event.into(), state);
    }

    pub fn on_mouse_event<T>(&self, event: T, state: &mut EditorState)
    where
        T: Into<MouseEvent>,
    {
        MouseEventHandler::on_event(event.into(), state);
    }
}

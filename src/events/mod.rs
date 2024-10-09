pub(crate) mod deprecated_input;
mod key;
#[cfg(feature = "mouse-support")]
pub(crate) mod mouse;

pub use key::{KeyEvent, KeyEventHandler, KeyEventRegister};

#[cfg(feature = "mouse-support")]
pub use mouse::{MouseEvent, MouseEventHandler};

use crate::EditorState;
use ratatui::crossterm::event::Event as CTEvent;

/// Handles key and mouse events.
#[derive(Default, Clone)]
pub struct EditorEventHandler {
    pub key_handler: KeyEventHandler,
}

impl EditorEventHandler {
    /// Creates a new `EditorEvent` handler.
    #[must_use]
    pub fn new(key_handler: KeyEventHandler) -> Self {
        Self { key_handler }
    }

    /// Handles key and mouse events.
    pub fn on_event<T>(&mut self, event: T, state: &mut EditorState)
    where
        T: Into<Event>,
    {
        match event.into() {
            Event::Key(event) => self.on_key_event(event, state),
            #[cfg(feature = "mouse-support")]
            Event::Mouse(event) => self.on_mouse_event(event, state),
            Event::None => (),
        }
    }

    /// Handles key events.
    pub fn on_key_event<T>(&mut self, event: T, state: &mut EditorState)
    where
        T: Into<KeyEvent>,
    {
        self.key_handler.on_event(event.into(), state);
    }

    #[cfg(feature = "mouse-support")]
    /// Handles mouse events.
    pub fn on_mouse_event<T>(&self, event: T, state: &mut EditorState)
    where
        T: Into<MouseEvent>,
    {
        MouseEventHandler::on_event(event.into(), state);
    }
}

pub enum Event {
    Key(KeyEvent),
    #[cfg(feature = "mouse-support")]
    Mouse(MouseEvent),
    None,
}

impl From<CTEvent> for Event {
    fn from(value: CTEvent) -> Self {
        match value {
            CTEvent::Key(event) => Self::Key(event.into()),
            #[cfg(feature = "mouse-support")]
            CTEvent::Mouse(event) => Self::Mouse(event.into()),
            _ => Self::None,
        }
    }
}

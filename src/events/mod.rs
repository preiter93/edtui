pub(crate) mod deprecated_input;
mod key;
#[cfg(feature = "mouse-support")]
pub(crate) mod mouse;
pub(crate) mod paste;

pub use key::{KeyEvent, KeyEventHandler, KeyEventRegister};

#[cfg(feature = "mouse-support")]
pub use mouse::{MouseEvent, MouseEventHandler};

use crate::{events::paste::PasteEventHandler, EditorState};
use crossterm::event::Event as CTEvent;

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
            Event::Paste(text) => self.on_paste_event(text, state),
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

    /// Handles paste events.
    pub fn on_paste_event(&self, text: String, state: &mut EditorState) {
        PasteEventHandler::on_event(text, state);
    }
}

pub enum Event {
    Key(KeyEvent),
    #[cfg(feature = "mouse-support")]
    Mouse(MouseEvent),
    Paste(String),
    None,
}

impl From<CTEvent> for Event {
    fn from(value: CTEvent) -> Self {
        match value {
            CTEvent::Key(event) => Self::Key(event.into()),
            #[cfg(feature = "mouse-support")]
            CTEvent::Mouse(event) => Self::Mouse(event.into()),
            CTEvent::Paste(text) => Self::Paste(text),
            _ => Self::None,
        }
    }
}

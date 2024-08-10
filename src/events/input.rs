//! Handles key input events
#![allow(deprecated)]
use crate::EditorState;

use super::key::{KeyEvent, KeyEventHandler};

#[derive(Clone, Debug)]
#[deprecated(since = "0.6.0", note = "Use EditorEvent instead.")]
pub struct EditorInput {
    event_handler: KeyEventHandler,
}

impl EditorInput {
    pub fn on_key<T>(&mut self, key: T, state: &mut EditorState)
    where
        T: Into<KeyEvent> + Copy,
    {
        self.event_handler.on_event(key, state);
    }
}

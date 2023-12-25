#![cfg(feature = "arboard")]
use super::{Clipboard, ClipboardTrait};
use crate::clipboard::InternalClipboard;
use std::error::Error;

pub struct ArboardClipboard {
    inner: arboard::Clipboard,
}

impl ArboardClipboard {
    /// Instantiates a new arboard clipboard.
    ///
    /// ## Errors
    /// - Platform not supported.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let inner = arboard::Clipboard::new()?;
        Ok(Self { inner })
    }
}

impl ClipboardTrait for ArboardClipboard {
    fn set_text(&mut self, text: String) {
        let _ = self.inner.set_text(text);
    }

    fn get_text(&mut self) -> String {
        self.inner.get_text().unwrap_or_default()
    }
}

#[cfg(feature = "arboard")]
impl Default for Clipboard {
    /// Creates a new `Clipboard` with `ArboardClipboard`.
    /// The arboard clipboard captures the systems clipboard.
    /// Falls back to internal clipboard if the arboard clipboard
    /// could not be instantiated.
    fn default() -> Self {
        if let Ok(clipboard) = ArboardClipboard::new() {
            return Self::new(clipboard);
        }
        Self::new(InternalClipboard::default())
    }
}

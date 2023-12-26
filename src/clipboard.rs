//! The editors clipboard
//!
//! ## Default clipboard
//!
//! `EdTUI` uses the arboard clipboard by default, which enables copy and paste
//! operations between the system clipboard and the editor. However, if a lighter
//! clipboard is desired, the "arboard" function flag can be deactivated. In this
//! case, the internal clipboard is used, which only supports copying and pasting
//! within the editor.
//!
//! ## Example: A custom global clipboard
//!
//! `EdTUI` allows you to set a custom clipboard. This is useful if you want to
//! define a global clipboard for example:
//!
//!```ignore
//! use once_cell::sync::Lazy;
//! use std::sync::Mutex;
//!
//! static CLIPBOARD: Lazy<GlobalClipboard> = Lazy::new(|| GlobalClipboard::new());
//!
//! struct GlobalClipboard(Mutex<Clipboard>);
//!
//! impl GlobalClipboard {
//!     pub fn new() -> Self {
//!         Self(Mutex::new(arboard::Clipboard::new().unwrap()))
//!     }
//! }
//!
//! impl ClipboardTrait for &GlobalClipboard {
//!     fn set_text(&mut self, text: String) {
//!         if let Ok(mut clipboard) = self.0.lock() {
//!             let _ = clipboard.set_text(text);
//!         }
//!     }
//!
//!     fn get_text(&mut self) -> String {
//!         if let Ok(mut clipboard) = self.0.lock() {
//!             return clipboard.get_text().unwrap_or_default();
//!         }
//!         String::new()
//!     }
//! }
//!
//! let mut state = EditorState::default();
//! state.set_clipboard(Lazy::force(&CLIPBOARD));
//!```
#[cfg(feature = "arboard")]
mod arboard;

use std::{cell::RefCell, rc::Rc};

/// Trait defining clipboard operations.
pub trait ClipboardTrait {
    /// Sets text to the clipboard.
    fn set_text(&mut self, text: String);

    /// Retrieves text from the clipboard.
    fn get_text(&mut self) -> String;
}

/// A clipboard for the editor.
///
/// This struct can hold any type that implements [`ClipboardTrait`].
#[derive(Clone)]
pub struct Clipboard(Rc<RefCell<dyn ClipboardTrait>>);

impl Clipboard {
    /// Creates a new `Clipboard` instance with a provided clipboard implementation.
    ///
    /// ## Example
    ///
    /// ```
    /// use edtui::clipboard::{Clipboard, ClipboardTrait};
    ///
    /// struct MyClipboard(String);
    ///
    /// impl ClipboardTrait for MyClipboard {
    ///     fn set_text(&mut self, text: String) {
    ///         self.0 = text;
    ///     }
    ///
    ///     fn get_text(&mut self) -> String {
    ///         self.0.clone()
    ///     }
    /// }
    ///
    /// let clipboard = MyClipboard(String::new());
    /// let clipboard_wrapper = Clipboard::new(clipboard);
    /// ```
    #[must_use]
    pub fn new(clipboard: impl ClipboardTrait + 'static) -> Self {
        Clipboard(Rc::new(RefCell::new(clipboard)))
    }
}

impl ClipboardTrait for Clipboard {
    fn set_text(&mut self, text: String) {
        self.0.borrow_mut().set_text(text);
    }

    fn get_text(&mut self) -> String {
        self.0.borrow_mut().get_text()
    }
}

#[derive(Default)]
pub struct InternalClipboard(String);

impl ClipboardTrait for InternalClipboard {
    fn set_text(&mut self, text: String) {
        self.0 = text;
    }

    fn get_text(&mut self) -> String {
        self.0.clone()
    }
}

#[cfg(not(feature = "arboard"))]
impl Default for Clipboard {
    /// Creates a new `Clipboard` with `InternalClipboard`.
    /// The internal clipboard does not capture the systems clipboard.
    fn default() -> Self {
        Self::new(InternalClipboard::default())
    }
}

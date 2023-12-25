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

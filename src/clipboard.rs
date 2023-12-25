use std::{cell::RefCell, error::Error, rc::Rc};

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

    /// Creates a new `Clipboard` with `InternalClipboard`.
    /// The internal clipboard does not capture the systems clipboard.
    #[must_use]
    pub fn internal() -> Self {
        Self::new(InternalClipboard::default())
    }

    /// Creates a new `Clipboard` with `ArboardClipboard`.
    /// The arboard clipboard captures the systems clipboard.
    /// Falls back to internal clipboard if the arboard clipboard
    /// could not be instantiated
    #[must_use]
    pub fn arboard() -> Self {
        if let Ok(clipboard) = ArboardClipboard::new() {
            return Self::new(clipboard);
        }
        Self::internal()
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

#[cfg(feature = "arboard")]
pub struct ArboardClipboard {
    inner: arboard::Clipboard,
}

#[cfg(feature = "arboard")]
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

pub mod mode;
pub mod selection;
mod undo;
mod view;

use self::view::ViewState;
use self::{mode::EditorMode, selection::Selection, undo::Stack};
use crate::clipboard::Clipboard;
use crate::{Index2, Lines};

/// Represents the state of an editor.
#[derive(Clone)]
pub struct EditorState {
    /// The text in the editor.
    pub lines: Lines,

    /// The current cursor position in the editor.
    pub cursor: Index2,

    /// The mode of the editor (insert, visual or normal mode).
    pub mode: EditorMode,

    /// Represents the selection in the editor, if any.
    pub selection: Option<Selection>,

    /// Internal view state of the editor.
    pub(crate) view: ViewState,

    /// Stack for undo operations.
    pub(crate) undo: Stack,

    /// Stack for redo operations.
    pub(crate) redo: Stack,

    /// Clipboard for yank and paste operations.
    pub(crate) clip: Clipboard,
}

impl Default for EditorState {
    /// Creates a default `EditorState` with no text.
    fn default() -> Self {
        EditorState::new(Lines::default())
    }
}

impl EditorState {
    /// Creates a new editor state.
    ///
    /// # Example
    ///
    /// ```
    /// use edtui::{EditorState, Lines};
    ///
    /// let state = EditorState::new(Lines::from("First line\nSecond Line"));
    /// ```
    #[must_use]
    pub fn new(lines: Lines) -> EditorState {
        EditorState {
            lines,
            cursor: Index2::new(0, 0),
            mode: EditorMode::Normal,
            selection: None,
            view: ViewState::default(),
            undo: Stack::new(),
            redo: Stack::new(),
            clip: Clipboard::default(),
        }
    }

    /// Set a custom clipboard.
    pub fn set_clipboard(&mut self, clipboard: Clipboard) {
        self.clip = clipboard;
    }
}

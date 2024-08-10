//! The editors state
pub mod mode;
mod search;
pub mod selection;
mod undo;
mod view;

use view::Offset;

use self::search::SearchState;
use self::view::ViewState;
use self::{mode::EditorMode, selection::Selection, undo::Stack};
use crate::clipboard::{Clipboard, ClipboardTrait};
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

    /// State holding the search results in search mode.
    pub(crate) search: SearchState,

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
            search: SearchState::default(),
            undo: Stack::new(),
            redo: Stack::new(),
            clip: Clipboard::default(),
        }
    }

    /// Set a custom clipboard.
    pub fn set_clipboard(&mut self, clipboard: impl ClipboardTrait + 'static) {
        self.clip = Clipboard::new(clipboard);
    }

    /// Sets the offset from the upper-left corner of the terminal window to the start of the editor buffer.
    ///
    /// There are two offsets involved in determining the mouse position relative to the editor text:
    ///
    /// 1. **window_to_editor_offset**: This offset is from the terminal window to the start of the editor
    /// area. This must be set manually by the user using this method.
    ///
    /// 2. **editor_to_textarea_offset**: This offset is from the start of the editor area to the actual textarea
    /// (e.g., borders, padding). This offset is set automatically.
    ///
    /// Both of these offsets are necessary to correctly calculate the mouse position in relation to the text
    /// within the editor.
    pub(crate) fn set_window_to_editor_offset<T: Into<Offset>>(&mut self, offset: T) {
        self.view.window_to_editor_offset = offset.into();
    }
}

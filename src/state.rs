//! The editors state
pub mod highlight;
pub mod mode;
mod search;
pub mod selection;
mod undo;
mod view;

use self::highlight::Highlight;
use self::search::SearchState;
use self::view::ViewState;
use self::{mode::EditorMode, selection::Selection, undo::Stack};
use crate::actions::Execute;
use crate::clipboard::{Clipboard, ClipboardTrait};
use crate::helper::max_col;
use crate::{Index2, Lines};
use ratatui_core::layout::Position;

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

    /// Custom highlight ranges with their styles.
    pub highlights: Vec<Highlight>,

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

    /// Flag indicating a system editor was requested.
    #[cfg(feature = "system-editor")]
    pub(crate) system_edit_requested: bool,
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
            highlights: Vec::new(),
            view: ViewState::default(),
            search: SearchState::default(),
            undo: Stack::new(),
            redo: Stack::new(),
            clip: Clipboard::default(),
            #[cfg(feature = "system-editor")]
            system_edit_requested: false,
        }
    }

    /// Execute an action on the editor state
    /// # Example
    ///
    /// ```
    /// use edtui::{EditorState, Lines};
    /// use edtui::actions::DeleteLine;
    ///
    /// let mut state = EditorState::new(Lines::from("Hello wold!"));
    /// state.execute(DeleteLine(1))
    /// ```
    pub fn execute(&mut self, mut action: impl Execute) {
        action.execute(self);
    }

    /// Set a custom clipboard.
    pub fn set_clipboard(&mut self, clipboard: impl ClipboardTrait + 'static) {
        self.clip = Clipboard::new(clipboard);
    }

    /// Returns the current search pattern.
    #[must_use]
    pub fn search_pattern(&self) -> String {
        self.search.pattern.clone()
    }

    /// Clamps the column of the cursor if the cursor is out of bounds.
    /// In normal or visual mode, clamps on `col = len() - 1`, in insert
    /// mode on `col = len()`.
    pub(crate) fn clamp_column(&mut self) {
        let max_col = max_col(&self.lines, &self.cursor, self.mode);
        self.cursor.col = self.cursor.col.min(max_col);
    }

    /// Returns the cursor's screen position, computed during the last render.
    ///
    /// This is the absolute position in terminal coordinates where the cursor
    /// should be displayed. It accounts for viewport scrolling, line wrapping,
    /// tab width, and the editor's position on screen.
    ///
    /// Returns `None` if the editor has not been rendered yet.
    #[must_use]
    pub fn cursor_screen_position(&self) -> Option<Position> {
        self.view.cursor_screen_position
    }

    /// Enables or disables single-line mode.
    ///
    /// When enabled, newline insertion is blocked. This is useful for search boxes,
    /// single-line input fields, and similar use cases.
    ///
    /// # Example
    ///
    /// ```
    /// use edtui::{EditorState, Lines};
    ///
    /// let mut state = EditorState::new(Lines::from("Search query"));
    /// state.set_single_line(true);
    /// ```
    pub fn set_single_line(&mut self, single_line: bool) {
        self.view.single_line = single_line;
    }

    /// Returns whether single-line mode is enabled.
    ///
    /// In single-line mode, newline insertion is blocked.
    #[must_use]
    pub fn is_single_line(&self) -> bool {
        self.view.single_line
    }

    /// Add a custom highlight range.
    pub fn add_highlight(&mut self, highlight: Highlight) {
        self.highlights.push(highlight);
    }

    /// Clear all custom highlights.
    pub fn clear_highlights(&mut self) {
        self.highlights.clear();
    }

    /// Set all highlights, replacing any existing ones.
    pub fn set_highlights(&mut self, highlights: Vec<Highlight>) {
        self.highlights = highlights;
    }

    /// Returns the current viewport offset as (x, y).
    ///
    /// The viewport offset represents the top-left corner of the visible area
    /// in editor coordinates. This is useful for implementing custom scroll
    /// behavior.
    ///
    /// - `x` is the horizontal offset (column)
    /// - `y` is the vertical offset (row)
    #[must_use]
    pub fn viewport_offset(&self) -> (usize, usize) {
        (self.view.viewport.x, self.view.viewport.y)
    }

    /// Sets the viewport offset to the specified (x, y) position.
    ///
    /// This allows manual control of the scroll position. The viewport offset
    /// represents the top-left corner of the visible area in editor coordinates.
    ///
    /// - `x` is the horizontal offset (column)
    /// - `y` is the vertical offset (row)
    ///
    /// Note: The viewport may be adjusted during the next render to keep the
    /// cursor visible, depending on the cursor position.
    pub fn set_viewport_offset(&mut self, x: usize, y: usize) {
        self.view.viewport.x = x;
        self.view.viewport.y = y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EditorView;
    use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};

    #[test]
    fn test_cursor_screen_position_after_render() {
        let mut state = EditorState::new(Lines::from("Hello World"));
        assert!(state.cursor_screen_position().is_none());

        let area = Rect::new(0, 0, 20, 5);
        let mut buffer = Buffer::empty(area);
        EditorView::new(&mut state).render(area, &mut buffer);

        let pos = state.cursor_screen_position();
        assert!(pos.is_some());

        let pos = pos.unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_cursor_screen_position_with_offset() {
        let mut state = EditorState::new(Lines::from("Hello World"));
        state.cursor = Index2::new(0, 5);

        let area = Rect::new(10, 5, 20, 5);
        let mut buffer = Buffer::empty(Rect::new(0, 0, 40, 20));
        EditorView::new(&mut state).render(area, &mut buffer);

        let pos = state.cursor_screen_position().unwrap();
        // Cursor column 5 + area.x (10) = 15
        assert_eq!(pos.x, 15);
        // Cursor row 0 + area.y (5) = 5
        assert_eq!(pos.y, 5);
    }

    #[test]
    fn test_cursor_screen_position_multiline() {
        let mut state = EditorState::new(Lines::from("Line 1\nLine 2\nLine 3"));
        state.cursor = Index2::new(2, 3);

        let area = Rect::new(0, 0, 20, 10);
        let mut buffer = Buffer::empty(area);
        EditorView::new(&mut state).render(area, &mut buffer);

        let pos = state.cursor_screen_position().unwrap();
        assert_eq!(pos.x, 3);
        assert_eq!(pos.y, 2);
    }

    #[test]
    fn test_single_line_mode_blocks_line_break() {
        use crate::actions::LineBreak;

        let mut state = EditorState::new(Lines::from("Hello World"));
        state.set_single_line(true);
        state.cursor = Index2::new(0, 5);

        LineBreak(1).execute(&mut state);

        // Line break should be blocked
        assert_eq!(state.lines, Lines::from("Hello World"));
        assert_eq!(state.cursor, Index2::new(0, 5));
    }

    #[test]
    fn test_single_line_mode_blocks_insert_newline_char() {
        use crate::actions::InsertChar;

        let mut state = EditorState::new(Lines::from("Hello"));
        state.set_single_line(true);
        state.cursor = Index2::new(0, 5);

        InsertChar('\n').execute(&mut state);

        // Newline char should be blocked
        assert_eq!(state.lines, Lines::from("Hello"));
    }

    #[test]
    fn test_single_line_mode_paste_replaces_newlines() {
        use crate::actions::Paste;
        use crate::clipboard::InternalClipboard;

        let mut state = EditorState::new(Lines::from("Hello"));
        state.set_clipboard(InternalClipboard::default());
        state.set_single_line(true);
        state.cursor = Index2::new(0, 5);

        // Paste text with newlines
        state.clip.set_text("Line1\nLine2\nLine3".to_string());
        Paste.execute(&mut state);

        // Newlines should be replaced with spaces
        assert_eq!(state.lines, Lines::from("HelloLine1 Line2 Line3"));
    }
}

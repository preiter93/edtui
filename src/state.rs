pub mod mode;
pub mod position;
pub mod selection;
mod undo;
mod view;

use self::view::ViewState;
use self::{mode::EditorMode, position::Position, selection::Selection, undo::Stack};
use crate::Lines;
use jagged::index::RowIndex;
use jagged::Jagged;

#[derive(Clone)]
pub struct EditorState {
    pub lines: Lines,
    pub cursor: Position,
    pub mode: EditorMode,
    pub selection: Option<Selection>,
    pub(crate) view: ViewState,
    pub(crate) undo: Stack,
    pub(crate) redo: Stack,
}

impl Default for EditorState {
    fn default() -> Self {
        EditorState::new(Jagged::default())
    }
}

impl EditorState {
    /// Create a new editor state.
    #[must_use]
    pub fn new(lines: Lines) -> EditorState {
        EditorState {
            lines,
            cursor: Position::new(0, 0),
            mode: EditorMode::Normal,
            selection: None,
            view: ViewState::default(),
            undo: Stack::new(),
            redo: Stack::new(),
        }
    }

    /// Get the number of rows.
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.lines.len()
    }

    /// Get the number of columns in the current line.
    #[must_use]
    pub(crate) fn len_col(&self) -> usize {
        self.len_col_at(self.cursor.line)
    }

    /// Get the number of columns of a line from the line index.
    #[must_use]
    pub(crate) fn len_col_at(&self, index: usize) -> usize {
        match self.lines.get(RowIndex::new(index)) {
            Some(line) => line.len(),
            None => 0,
        }
    }

    /// Set the cursor position to the specified line and column
    pub(crate) fn set_cursor_position(&mut self, line: usize, column: usize) {
        self.cursor.line = line;
        self.cursor.column = column;
    }

    /// Set the selection to the specified start and end positions
    pub(crate) fn set_selection(&mut self, start: Position, end: Position) {
        self.selection = Some(Selection { start, end });
    }

    /// Set the selection start positions
    pub(crate) fn set_selection_start(&mut self, start: Position) {
        if let Some(end) = self.selection.as_ref().map(|x| x.end) {
            self.selection = Some(Selection { start, end });
        }
    }

    /// Set the selection end positions
    pub(crate) fn set_selection_end(&mut self, end: Position) {
        if let Some(start) = self.selection.as_ref().map(|x| x.start) {
            self.selection = Some(Selection { start, end });
        }
    }

    /// Skip whitespaces moving to the right. Stop at the end of the line.
    pub(crate) fn skip_whitespace(&mut self) {
        let start = self.cursor.as_index();
        let mut index = start;

        if let Some(line) = self.lines.get(RowIndex::new(start.row)) {
            for (i, &ch) in line.iter().enumerate().skip(start.col) {
                if !ch.is_ascii_whitespace() {
                    index.col = i;
                    break;
                }
            }
        }

        self.cursor = index.into();
    }

    /// Skip whitespaces moving to the left. Stop at the start of the line.
    pub(crate) fn skip_whitespace_rev(&mut self) {
        let start = self.cursor.as_index();
        let mut index = start;

        if let Some(line) = self.lines.get(RowIndex::new(start.row)) {
            let skip = line.len().saturating_sub(start.col + 1);
            for &ch in line.iter().rev().skip(skip) {
                if !ch.is_ascii_whitespace() {
                    break;
                }
                index.col = index.col.saturating_sub(1);
            }
        }

        self.cursor = index.into();
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_whitespace() {
        let mut state = EditorState::new(Lines::from("  World!"));

        state.skip_whitespace();
        assert_eq!(state.cursor.column, 2);

        state.skip_whitespace();
        assert_eq!(state.cursor.column, 2);
    }

    #[test]
    fn test_skip_whitespace_rev() {
        let mut state = EditorState::new(Lines::from("  x World!"));
        state.set_cursor_position(0, 3);

        state.skip_whitespace_rev();
        assert_eq!(state.cursor.column, 2);

        state.skip_whitespace_rev();
        assert_eq!(state.cursor.column, 2);

        state.set_cursor_position(0, 1);
        state.skip_whitespace_rev();
        assert_eq!(state.cursor.column, 0);
    }
}

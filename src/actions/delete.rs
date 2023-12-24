use jagged::index::RowIndex;

use super::Execute;
use crate::{helper::len_col, EditorMode, EditorState};

/// Deletes a character at the current cursor position. Does not
/// move the cursor position unless it is at the end of the line
#[derive(Clone, Debug, Copy)]
pub struct RemoveChar(pub usize);

impl Execute for RemoveChar {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        for _ in 0..self.0 {
            if len_col(&state) == 0 {
                break;
            }
            let _ = state.lines.remove(state.cursor.as_index());
            state.cursor.column = state.cursor.column.min(len_col(&state).saturating_sub(1));
        }
    }
}

/// Deletes a character to the left of the current cursor. Deletes
/// the line break if the the cursor is in column zero.
#[derive(Clone, Debug, Copy)]
pub struct DeleteChar(pub usize);

impl Execute for DeleteChar {
    fn execute(&mut self, state: &mut EditorState) {
        fn move_left(state: &mut EditorState) {
            if state.cursor.column > 0 {
                state.cursor.column -= 1;
            } else if state.cursor.line > 0 {
                state.cursor.line -= 1;
                state.cursor.column = len_col(&state);
            }
        }

        state.capture();
        for _ in 0..self.0 {
            if state.cursor.column == 0 && state.cursor.line == 0 {
                break;
            }

            if state.cursor.column == 0 {
                let mut rest = state.lines.split_off(state.cursor.as_index());
                move_left(state);
                state.lines.merge(&mut rest);
            } else {
                move_left(state);
                let _ = state.lines.remove(state.cursor.as_index());
            }
        }
    }
}

/// Deletes the current line.
#[derive(Clone, Debug, Copy)]
pub struct DeleteLine(pub usize);

impl Execute for DeleteLine {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        for _ in 0..self.0 {
            if state.cursor.line >= state.lines.len() {
                break;
            }
            state.lines.remove(RowIndex::new(state.cursor.line));
            state.cursor.column = 0;
            state.cursor.line = state.cursor.line.min(state.lines.len().saturating_sub(1));
        }
    }
}

/// Deletes the current selection.
#[derive(Clone, Debug, Copy)]
pub struct DeleteSelection;

impl Execute for DeleteSelection {
    // TODO: Implement a better way to delete a selection,
    // possibly using a drain iterator.
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(selection) = state.selection.take() {
            state.cursor = selection.end();
            RemoveChar(1).execute(state);
            while state.cursor != selection.start() {
                DeleteChar(1).execute(state);
            }
        }
        state.selection = None;
        state.mode = EditorMode::Normal;
    }
}

#[cfg(test)]
mod tests {
    use crate::{state::position::Position, Lines};

    use super::*;
    fn test_state() -> EditorState {
        EditorState::new(Lines::from("Hello World!\n\n123."))
    }

    #[test]
    fn test_remove() {
        let mut state = test_state();

        state.cursor = Position::new(0, 4);
        RemoveChar(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hell World!\n\n123."));

        state.cursor = Position::new(0, 10);
        RemoveChar(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 9));
        assert_eq!(state.lines, Lines::from("Hell World\n\n123."));
    }

    #[test]
    fn test_delete_char() {
        let mut state = test_state();

        state.cursor = Position::new(0, 5);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hell World!\n\n123."));

        state.cursor = Position::new(0, 11);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 10));
        assert_eq!(state.lines, Lines::from("Hell World\n\n123."));
    }
}

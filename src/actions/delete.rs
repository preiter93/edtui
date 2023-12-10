use jagged::index::RowIndex;

use super::{Execute, SwitchMode};
use crate::{EditorMode, EditorState};

/// Deletes a character at the current cursor position. Does not
/// move the cursor position unless it is at the end of the line
#[derive(Clone, Debug, Copy)]
pub struct Remove(pub usize);

impl Execute for Remove {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        for _ in 0..self.0 {
            if state.len_col() == 0 {
                break;
            }
            let _ = state.lines.remove(state.cursor.as_index());
            state.cursor.column = state.cursor.column.min(state.len_col().saturating_sub(1));
        }
    }
}

/// Deletes a character to the left of the current cursor. Does not
/// move the cursor position unless it is at the end of the line.
#[derive(Clone, Debug, Copy)]
pub struct DeleteChar(pub usize);

impl Execute for DeleteChar {
    fn execute(&mut self, state: &mut EditorState) {
        fn move_left(state: &mut EditorState) {
            if state.cursor.column > 0 {
                state.cursor.column -= 1;
            } else if state.cursor.line > 0 {
                state.cursor.line -= 1;
                state.cursor.column = state.len_col();
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
                Remove(1).execute(state);
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
            if state.cursor.line >= state.len() {
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
    // TODO: Implement a better way to delete selection
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(selection) = state.selection.take() {
            state.cursor = selection.end();
            Remove(1).execute(state);
            while state.cursor != selection.start() {
                DeleteChar(1).execute(state);
            }
        }
        state.clear_selection();
        SwitchMode(EditorMode::Normal).execute(state);
    }
}

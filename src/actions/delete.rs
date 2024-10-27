use jagged::index::RowIndex;

use super::Execute;
use crate::{
    clipboard::ClipboardTrait,
    helper::{is_out_of_bounds, max_col_insert},
    state::selection::Selection,
    EditorMode, EditorState, Index2, Lines,
};

/// Deletes a character at the current cursor position. Does not
/// move the cursor position unless it is at the end of the line
#[derive(Clone, Debug, Copy)]
pub struct RemoveChar(pub usize);

impl Execute for RemoveChar {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        state.clamp_column();
        for _ in 0..self.0 {
            let lines = &mut state.lines;
            let index = &mut state.cursor;
            if lines.len_col(index.row).unwrap_or_default() == 0 {
                return;
            }
            let _ = lines.remove(*index);
            index.col = index.col.min(
                lines
                    .len_col(index.row)
                    .unwrap_or_default()
                    .saturating_sub(1),
            );
        }
    }
}

/// Replaces the character under the cursor with a given character.
#[derive(Clone, Debug, Copy)]
pub struct ReplaceChar(pub char);

impl Execute for ReplaceChar {
    fn execute(&mut self, state: &mut EditorState) {
        let index = state.cursor;
        if is_out_of_bounds(&state.lines, &index) {
            return;
        }
        state.capture();
        if let Some(ch) = state.lines.get_mut(index) {
            *ch = self.0;
        };
    }
}

/// Deletes a character to the left of the current cursor. Deletes
/// the line break if the the cursor is in column zero.
#[derive(Clone, Debug, Copy)]
pub struct DeleteChar(pub usize);

impl Execute for DeleteChar {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        for _ in 0..self.0 {
            delete_char(&mut state.lines, &mut state.cursor);
        }
    }
}

fn delete_char(lines: &mut Lines, index: &mut Index2) {
    fn move_left(lines: &Lines, index: &mut Index2) {
        if index.col > 0 {
            index.col -= 1;
        } else if index.row > 0 {
            index.row -= 1;
            index.col = lines.len_col(index.row).unwrap_or_default();
        }
    }

    let len_col = lines.len_col(index.row).unwrap_or_default();
    if len_col == 0 && index.row == 0 {
        return;
    }

    if index.col > len_col {
        index.col = len_col;
    }

    if index.col == 0 {
        let mut rest = lines.split_off(*index);
        move_left(lines, index);
        lines.merge(&mut rest);
    } else {
        let max_col = max_col_insert(lines, index);
        index.col = index.col.min(max_col);
        move_left(lines, index);
        let _ = lines.remove(*index);
    }
}

/// Deletes the current line.
#[derive(Clone, Debug, Copy)]
pub struct DeleteLine(pub usize);

impl Execute for DeleteLine {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        for _ in 0..self.0 {
            if state.cursor.row >= state.lines.len() {
                break;
            }
            let row_index = RowIndex::new(state.cursor.row);
            let deleted_line = state.lines.remove(row_index).iter().collect::<String>();
            state.clip.set_text(String::from('\n') + &deleted_line);
            state.cursor.col = 0;
            state.cursor.row = state.cursor.row.min(state.lines.len().saturating_sub(1));
        }
    }
}

/// Deletes from the current cursor position to the end of the line
#[derive(Clone, Debug, Copy)]
pub struct DeleteToEndOfLine;

impl Execute for DeleteToEndOfLine {
    fn execute(&mut self, state: &mut EditorState) {
        if is_out_of_bounds(&state.lines, &state.cursor) {
            return;
        }
        state.capture();
        let Some(row) = state.lines.get_mut(RowIndex::new(state.cursor.row)) else {
            return;
        };
        let deleted_chars = row.drain(state.cursor.col..);
        state.cursor.col = state.cursor.col.saturating_sub(1);
        state.clip.set_text(deleted_chars.collect());
    }
}

/// Deletes the current selection.
#[derive(Clone, Debug)]
pub struct DeleteSelection;

impl Execute for DeleteSelection {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(selection) = state.selection.take() {
            state.capture();
            let drained = delete_selection(state, &selection);
            state.clip.set_text(drained.into());
        }
        state.selection = None;
        state.mode = EditorMode::Normal;
    }
}

pub(crate) fn delete_selection(state: &mut EditorState, selection: &Selection) -> Lines {
    state.cursor = selection.start();
    state.clamp_column();
    selection.extract_from(&mut state.lines)
}

/// Joins line below to the current line.
#[derive(Clone, Debug, Copy)]
pub struct JoinLineWithLineBelow;

impl Execute for JoinLineWithLineBelow {
    fn execute(&mut self, state: &mut EditorState) {
        if state.cursor.row + 1 >= state.lines.len() {
            return;
        }
        state.capture();
        state.lines.join_lines(state.cursor.row);
    }
}

#[cfg(test)]
mod tests {
    use crate::state::selection::Selection;
    use crate::Index2;
    use crate::Lines;

    use super::*;
    fn test_state() -> EditorState {
        EditorState::new(Lines::from("Hello World!\n\n123."))
    }

    #[test]
    fn test_remove_char() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 4);
        RemoveChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hell World!\n\n123."));

        state.cursor = Index2::new(0, 10);
        RemoveChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 9));
        assert_eq!(state.lines, Lines::from("Hell World\n\n123."));
    }

    #[test]
    fn test_replace_char() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 4);
        ReplaceChar('x').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hellx World!\n\n123."));

        // do nothing on empty line
        state.cursor = Index2::new(1, 0);
        ReplaceChar('x').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
        assert_eq!(state.lines, Lines::from("Hellx World!\n\n123."));

        // do nothing if out of bounds
        state.cursor = Index2::new(99, 0);
        ReplaceChar('x').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(99, 0));
        assert_eq!(state.lines, Lines::from("Hellx World!\n\n123."));
    }

    #[test]
    fn test_delete_char() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 5);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hell World!\n\n123."));

        state.cursor = Index2::new(0, 11);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));
        assert_eq!(state.lines, Lines::from("Hell World\n\n123."));
    }

    #[test]
    fn test_delete_char_empty_line() {
        let mut state = test_state();
        state.cursor = Index2::new(1, 99);

        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 12));
        assert_eq!(state.lines, Lines::from("Hello World!\n123."));

        let mut state = EditorState::new(Lines::from("\nb"));
        state.cursor = Index2::new(0, 1);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.lines, Lines::from("\nb"));
    }

    #[test]
    fn test_delete_line() {
        let mut state = test_state();
        state.cursor = Index2::new(2, 3);

        DeleteLine(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
        assert_eq!(state.lines, Lines::from("Hello World!\n"));

        DeleteLine(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from("Hello World!"));

        DeleteLine(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from(""));
    }

    #[test]
    fn test_delete_to_end_line() {
        let mut state = test_state();
        state.cursor = Index2::new(0, 3);

        DeleteToEndOfLine.execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 2));
        assert_eq!(state.lines, Lines::from("Hel\n\n123."));
    }

    #[test]
    fn test_delete_selection() {
        let mut state = test_state();
        let st = Index2::new(0, 1);
        let en = Index2::new(2, 0);
        state.selection = Some(Selection::new(st, en));

        DeleteSelection.execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.lines, Lines::from("H23."));
    }

    #[test]
    fn test_delete_selection_out_of_bounds() {
        let mut state = EditorState::new(Lines::from("123.\nHello World!\n456."));
        let st = Index2::new(0, 5);
        let en = Index2::new(2, 10);
        state.selection = Some(Selection::new(st, en));

        DeleteSelection.execute(&mut state);
        // assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.lines, Lines::from("123."));
    }
}

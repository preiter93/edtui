use super::Execute;
use crate::{state::selection::Selection, EditorMode, EditorState, Index2};

/// Selects text between specified delimiter characters.
///
/// It searches for the first occurrence of a delimiter character in the text to
/// define the start of the selection, and the next occurrence of any of the delimiter
/// characters to define the end of the selection.
#[derive(Clone, Debug, Copy)]
pub struct SelectBetween(pub char);

impl Execute for SelectBetween {
    fn execute(&mut self, state: &mut EditorState) {
        let cursor = state.cursor;
        let mut start: Option<Index2> = None;
        let mut end: Option<Index2> = None;
        let mut prev = cursor;
        for (value, index) in state.lines.iter().from(cursor) {
            if let Some(&c) = value {
                if c == self.0 {
                    end = Some(prev);
                    break;
                }
            }
            prev = index;
        }
        prev = cursor;
        for (value, index) in state.lines.iter().from(cursor).rev() {
            if let Some(&c) = value {
                if c == self.0 {
                    start = Some(prev);
                    break;
                }
            }
            prev = index;
        }
        if let (Some(start), Some(end)) = (start, end) {
            state.selection = Some(Selection { start, end });
            state.mode = EditorMode::Visual;
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct SelectLine;

impl Execute for SelectLine {
    fn execute(&mut self, state: &mut EditorState) {
        let row = state.cursor.row;
        if let Some(len_col) = state.lines.len_col(row) {
            let start = Index2::new(row, 0);
            let end = Index2::new(row, len_col.saturating_sub(1));
            state.selection = Some(Selection::new(start, end));
            state.mode = EditorMode::Visual;
        }
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
    fn test_select_line() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 4);
        SelectLine.execute(&mut state);
        assert_eq!(
            state.selection,
            Some(Selection {
                start: Index2::new(0, 0),
                end: Index2::new(0, 11),
            })
        );
        assert_eq!(state.mode, EditorMode::Visual);
        assert_eq!(state.cursor, Index2::new(0, 4));
    }
}

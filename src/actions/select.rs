use super::{delete::delete_selection, Execute};
use crate::{
    clipboard::ClipboardTrait, state::selection::Selection, EditorMode, EditorState, Index2, Lines,
};

/// Selects text between specified delimiter characters.
///
/// It searches for the first occurrence of a delimiter character in the text to
/// define the start of the selection, and the next occurrence of any of the delimiter
/// characters to define the end of the selection.
#[derive(Clone, Debug, Copy)]
pub struct SelectInnerBetween {
    opening: char,
    closing: char,
}

impl SelectInnerBetween {
    #[must_use]
    pub fn new(opening: char, closing: char) -> Self {
        Self { opening, closing }
    }
}

impl Execute for SelectInnerBetween {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(selection) =
            select_inner_between(&state.lines, state.cursor, self.opening, self.closing)
        {
            state.selection = Some(selection);
            state.mode = EditorMode::Visual;
        }
    }
}

fn select_inner_between(
    lines: &Lines,
    cursor: Index2,
    opening: char,
    closing: char,
) -> Option<Selection> {
    let mut start: Option<Index2> = None;
    let mut end: Option<Index2> = None;
    let mut prev = cursor;
    for (value, index) in lines.iter().from(cursor) {
        if let Some(&c) = value {
            if c == closing {
                end = Some(prev);
                break;
            }
        }
        prev = index;
    }
    prev = cursor;
    for (value, index) in lines.iter().from(cursor).rev() {
        if let Some(&c) = value {
            if c == opening {
                start = Some(prev);
                break;
            }
        }
        prev = index;
    }

    if let (Some(start), Some(end)) = (start, end) {
        return Some(Selection::new(start, end));
    }
    None
}

/// Changes text between specified delimiter characters.
#[derive(Clone, Debug, Copy)]
pub struct ChangeInnerBetween {
    opening: char,
    closing: char,
}

impl ChangeInnerBetween {
    #[must_use]
    pub fn new(opening: char, closing: char) -> Self {
        Self { opening, closing }
    }
}

impl Execute for ChangeInnerBetween {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(selection) =
            select_inner_between(&state.lines, state.cursor, self.opening, self.closing)
        {
            state.capture();
            let deleted = delete_selection(state, &selection);
            state.clip.set_text(deleted.into());
            state.mode = EditorMode::Insert;
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
            state.selection = Some(Selection::new(start, end).line_mode());
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
            Some(Selection::new(Index2::new(0, 0), Index2::new(0, 11),).line_mode())
        );
        assert_eq!(state.mode, EditorMode::Visual);
        assert_eq!(state.cursor, Index2::new(0, 4));
    }
}

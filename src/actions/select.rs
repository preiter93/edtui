use jagged::index::RowIndex;

use super::{delete::delete_selection, motion::CharacterClass, Execute};
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
        if let Some(selection) = select_between(
            &state.lines,
            state.cursor,
            |(ch, _)| *ch == self.opening,
            |(ch, _)| *ch == self.closing,
            |(_, _)| false,
            |(_, _)| false,
        ) {
            state.selection = Some(selection);
            state.mode = EditorMode::Visual;
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct SelectInnerWord;

impl Execute for SelectInnerWord {
    fn execute(&mut self, state: &mut EditorState) {
        let row_index = state.cursor.row;
        let Some(line) = state.lines.get(RowIndex::new(row_index)) else {
            return;
        };

        let Some(len_col) = state.lines.len_col(state.cursor.row) else {
            return;
        };

        let max_col_index = len_col.saturating_sub(1);

        let start_col = state.cursor.col;
        let start_char_class = CharacterClass::from(line.get(start_col));

        let opening_predicate =
            |(ch, _): (&char, usize)| CharacterClass::from(ch) != start_char_class.clone();
        let closing_predicate =
            |(ch, _): (&char, usize)| CharacterClass::from(ch) != start_char_class.clone();

        if let Some(selection) = select_between(
            &state.lines,
            state.cursor,
            opening_predicate,
            closing_predicate,
            |(_, col)| col == 0,
            |(_, col)| col == max_col_index,
        ) {
            state.selection = Some(selection);
        }
    }
}

fn select_between(
    lines: &Lines,
    cursor: Index2,
    opening_predicate_excl: impl Fn((&char, usize)) -> bool,
    closing_predicate_excl: impl Fn((&char, usize)) -> bool,
    opening_predicate_incl: impl Fn((&char, usize)) -> bool,
    closing_predicate_incl: impl Fn((&char, usize)) -> bool,
) -> Option<Selection> {
    let len_col = lines.len_col(cursor.row)?;
    if cursor.col >= len_col {
        return None;
    }

    let row_index = cursor.row;
    let line = lines.get(RowIndex::new(row_index))?;

    let start_col = cursor.col;

    let mut opening: Option<usize> = None;
    let mut prev_col = start_col;
    for col in (0..=start_col).rev() {
        if let Some(ch) = line.get(col) {
            if opening_predicate_excl((ch, col)) {
                opening = Some(prev_col);
                break;
            }
            if opening_predicate_incl((ch, col)) {
                opening = Some(col);
                break;
            }
        }
        prev_col = col;
    }

    let mut closing: Option<usize> = None;
    let mut prev_col = start_col;
    for col in start_col..len_col {
        if let Some(ch) = line.get(col) {
            if closing_predicate_excl((ch, col)) {
                closing = Some(prev_col);
                break;
            }
            if closing_predicate_incl((ch, col)) {
                closing = Some(col);
                break;
            }
        }
        prev_col = col;
    }

    if let (Some(opening), Some(closing)) = (opening, closing) {
        let selection = Selection::new(
            Index2::new(row_index, opening),
            Index2::new(row_index, closing),
        );
        Some(selection)
    } else {
        None
    }
}

#[derive(Clone, Debug, Copy)]
pub struct ChangeInnerWord;

impl Execute for ChangeInnerWord {
    fn execute(&mut self, state: &mut EditorState) {
        SelectInnerWord.execute(state);
        if let Some(selection) = state.selection.take() {
            state.capture();
            let deleted = delete_selection(state, &selection);
            state.clip.set_text(deleted.into());
        }
    }
}

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
        SelectInnerBetween::new(self.opening, self.closing).execute(state);
        if let Some(selection) = state.selection.take() {
            state.capture();
            let deleted = delete_selection(state, &selection);
            state.clip.set_text(deleted.into());
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

#[derive(Clone, Debug, Copy)]
pub struct ChangeSelection;
impl Execute for ChangeSelection {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(selection) = state.selection.take() {
            state.capture();
            let deleted = delete_selection(state, &selection);
            state.clip.set_text(deleted.into());
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

        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.line_mode);
        assert!(selection.contains_row(0));
        assert_eq!(state.mode, EditorMode::Visual);
        assert_eq!(state.cursor, Index2::new(0, 4));
    }

    #[test]
    fn test_select_line_with_motion() {
        use crate::actions::motion::MoveUp;
        use crate::EditorMode;

        let mut state = EditorState::new(Lines::from("First line\nSecond line\nThird line"));

        // Start on line 1, select line
        state.cursor = Index2::new(1, 5);
        SelectLine.execute(&mut state);

        // Should have line 1 selected
        let want = Some(Selection::new(Index2::new(1, 0), Index2::new(1, 10)).line_mode());
        assert_eq!(state.selection, want);

        // Move up to line 0 - should extend selection to cover both lines
        state.mode = EditorMode::Visual;
        MoveUp(1).execute(&mut state);

        // Should now select from line 0 to line 1 (complete lines)
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.contains_row(0));
        assert!(selection.contains_row(1));
        assert_eq!(state.cursor.row, 0);
    }

    #[test]
    fn test_select_line_with_downward_motion() {
        use crate::actions::motion::MoveDown;
        use crate::EditorMode;

        let mut state = EditorState::new(Lines::from("First line\nSecond line\nThird line"));

        // Start on line 1, select line
        state.cursor = Index2::new(1, 5);
        SelectLine.execute(&mut state);

        // Should have line 1 selected
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.line_mode);
        assert!(selection.contains_row(1));

        // Move down to line 2 - should extend selection to cover both lines
        state.mode = EditorMode::Visual;
        MoveDown(1).execute(&mut state);

        // Should now select from line 1 to line 2 (complete lines)
        // Should select from line 1 to line 2
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.contains_row(1));
        assert!(selection.contains_row(2));
        assert_eq!(state.cursor.row, 2);
    }

    #[test]
    fn test_select_line_with_horizontal_motion() {
        use crate::actions::motion::MoveForward;
        use crate::EditorMode;

        let mut state = EditorState::new(Lines::from("First line\nSecond line\nThird line"));

        // Start on line 1, select line
        state.cursor = Index2::new(1, 5);
        SelectLine.execute(&mut state);

        // Should have line 1 selected
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.line_mode);
        assert!(selection.contains_row(1));

        // Move forward horizontally - should still select the same complete line
        state.mode = EditorMode::Visual;
        MoveForward(3).execute(&mut state);

        // Should still select the same complete line (line mode ignores horizontal movement within same line)
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.contains_row(1));
        assert!(!selection.contains_row(0));
        assert!(!selection.contains_row(2));
        assert_eq!(state.cursor.row, 1);
        assert_eq!(state.cursor.col, 8); // cursor moved but selection stays on complete line
    }

    #[test]
    fn test_select_line_multiple_upward_movements() {
        use crate::actions::motion::MoveUp;
        use crate::EditorMode;

        let mut state = EditorState::new(Lines::from("Line 0\nLine 1\nLine 2\nLine 3"));

        // Start on line 2, select line
        state.cursor = Index2::new(2, 3);
        SelectLine.execute(&mut state);

        // Should have line 2 selected
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.line_mode);
        assert!(selection.contains_row(2));

        // Move up to line 1 - should extend selection
        state.mode = EditorMode::Visual;
        MoveUp(1).execute(&mut state);

        // Should select from line 1 to line 2
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.contains_row(1));
        assert!(selection.contains_row(2));
        assert_eq!(state.cursor.row, 1);

        // Move up again to line 0 - should extend selection further
        MoveUp(1).execute(&mut state);

        // Should now select from line 0 to line 2 (original anchor preserved)
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.contains_row(0));
        assert!(selection.contains_row(1));
        assert!(selection.contains_row(2));
        assert_eq!(state.cursor.row, 0);
    }

    #[test]
    fn test_select_line_multiple_downward_movements() {
        use crate::actions::motion::MoveDown;
        use crate::EditorMode;

        let mut state = EditorState::new(Lines::from("Line 0\nLine 1\nLine 2\nLine 3"));

        // Start on line 1, select line
        state.cursor = Index2::new(1, 3);
        SelectLine.execute(&mut state);

        // Should have line 1 selected
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.line_mode);
        assert!(selection.contains_row(1));

        // Move down to line 2 - should extend selection
        state.mode = EditorMode::Visual;
        MoveDown(1).execute(&mut state);

        // Should select from line 1 to line 2
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.contains_row(1));
        assert!(selection.contains_row(2));
        assert_eq!(state.cursor.row, 2);

        // Move down again to line 3 - should extend selection further
        MoveDown(1).execute(&mut state);

        // Should now select from line 1 to line 3 (original anchor preserved)
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.contains_row(1));
        assert!(selection.contains_row(2));
        assert!(selection.contains_row(3));
        assert_eq!(state.cursor.row, 3);
    }

    #[test]
    fn test_select_line_mixed_movements() {
        use crate::actions::motion::{MoveDown, MoveUp};
        use crate::EditorMode;

        let mut state = EditorState::new(Lines::from("Line 0\nLine 1\nLine 2\nLine 3\nLine 4"));

        // Start on line 2, select line
        state.cursor = Index2::new(2, 3);
        SelectLine.execute(&mut state);

        // Move up to line 1
        // Move up to line 1 - should extend selection to cover both lines
        state.mode = EditorMode::Visual;
        MoveUp(1).execute(&mut state);

        // Should select from line 1 to line 2 (complete lines)
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.contains_row(1));
        assert!(selection.contains_row(2));
        assert!(!selection.contains_row(0));
        assert!(!selection.contains_row(3));

        // Now move back down to line 2 - should shrink back to original
        MoveDown(1).execute(&mut state);

        // Should shrink back to just line 2 (original anchor)
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.contains_row(2));
        assert!(!selection.contains_row(1));
        assert!(!selection.contains_row(3));
        assert_eq!(state.cursor.row, 2);

        // Now move down to line 3 - should extend downward from anchor
        MoveDown(1).execute(&mut state);

        // Should select from line 2 to line 3
        assert!(state.selection.is_some());
        let selection = state.selection.as_ref().unwrap();
        assert!(selection.contains_row(2));
        assert!(selection.contains_row(3));
        assert!(!selection.contains_row(1));
        assert_eq!(state.cursor.row, 3);
    }

    #[test]
    fn test_select_inner_between() {
        let lines = Lines::from("\"Hello\" World");
        let mut state = EditorState::new(lines);
        state.cursor = Index2::new(0, 1);

        SelectInnerBetween::new('"', '"').execute(&mut state);

        let want = Selection::new(Index2::new(0, 1), Index2::new(0, 5));
        assert_eq!(state.selection.unwrap(), want);
    }

    #[test]
    fn test_select_inner_word() {
        let lines = Lines::from("Hello World");
        let mut state = EditorState::new(lines);
        state.cursor = Index2::new(0, 1);

        SelectInnerWord.execute(&mut state);

        let want = Selection::new(Index2::new(0, 0), Index2::new(0, 4));
        assert_eq!(state.selection.unwrap(), want);
    }
}

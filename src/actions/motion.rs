use std::cmp::min;

use crate::{
    helper::{find_matching_bracket, skip_empty_lines},
    state::selection::set_selection_with_lines,
};
use jagged::Index2;

use super::Execute;
use crate::{
    helper::{max_col, max_col_normal, skip_whitespace, skip_whitespace_rev},
    EditorMode, EditorState,
};

#[derive(Clone, Debug, Copy)]
pub struct MoveForward(pub usize);

impl Execute for MoveForward {
    fn execute(&mut self, state: &mut EditorState) {
        for _ in 0..self.0 {
            if state.cursor.col >= max_col(&state.lines, &state.cursor, state.mode) {
                break;
            }
            state.cursor.col += 1;
        }
        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveBackward(pub usize);

impl Execute for MoveBackward {
    fn execute(&mut self, state: &mut EditorState) {
        for _ in 0..self.0 {
            if state.cursor.col == 0 {
                break;
            }
            let max_col = max_col(&state.lines, &state.cursor, state.mode);
            if state.cursor.col > max_col {
                state.cursor.col = max_col;
            }
            state.cursor.col = state.cursor.col.saturating_sub(1);
        }
        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveUp(pub usize);

impl Execute for MoveUp {
    fn execute(&mut self, state: &mut EditorState) {
        for _ in 0..self.0 {
            if state.cursor.row == 0 {
                break;
            }
            state.cursor.row = state.cursor.row.saturating_sub(1);
        }
        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveDown(pub usize);

impl Execute for MoveDown {
    fn execute(&mut self, state: &mut EditorState) {
        for _ in 0..self.0 {
            if state.cursor.row >= state.lines.len().saturating_sub(1) {
                break;
            }
            state.cursor.row += 1;
        }
        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

/// Move one word forward. Breaks on the first character that is not of
/// the same class as the initial character or breaks on line ending.
/// Furthermore, after the first break, whitespaces are skipped.
#[derive(Clone, Debug, Copy)]
pub struct MoveWordForward(pub usize);

impl Execute for MoveWordForward {
    fn execute(&mut self, state: &mut EditorState) {
        if state.lines.is_empty() {
            return;
        }

        state.clamp_column();

        for _ in 0..self.0 {
            move_word_forward(state);
        }

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

fn move_word_forward(state: &mut EditorState) {
    let start_char_class = CharacterClass::from(state.lines.get(state.cursor));

    let start_index = match (
        state.lines.is_last_col(state.cursor),
        state.lines.is_last_row(state.cursor),
    ) {
        (true, true) => return,
        (true, false) => {
            state.cursor = Index2::new(state.cursor.row.saturating_add(1), 0);
            return;
        }
        _ => Index2::new(state.cursor.row, state.cursor.col.saturating_add(1)),
    };

    for (next_char, index) in state.lines.iter().from(start_index) {
        if CharacterClass::from(next_char) != start_char_class {
            state.cursor = index;
            skip_whitespace(&state.lines, &mut state.cursor);
            return;
        }
    }

    let max_col = max_col(&state.lines, &state.cursor, state.mode);
    state.cursor = Index2::new(state.cursor.row, max_col);
}

/// Move one word forward to the end of the word.
#[derive(Clone, Debug, Copy)]
pub struct MoveWordForwardToEndOfWord(pub usize);
impl Execute for MoveWordForwardToEndOfWord {
    fn execute(&mut self, state: &mut EditorState) {
        if state.lines.is_empty() {
            return;
        }

        state.clamp_column();

        for _ in 0..self.0 {
            move_word_forward_to_end_of_word(state);
        }

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

fn move_word_forward_to_end_of_word(state: &mut EditorState) {
    let mut start_index = match (
        state.lines.is_last_col(state.cursor),
        state.lines.is_last_row(state.cursor),
    ) {
        (true, true) => return,
        (true, false) => Index2::new(state.cursor.row.saturating_add(1), 0),
        _ => Index2::new(state.cursor.row, state.cursor.col.saturating_add(1)),
    };
    skip_empty_lines(&state.lines, &mut start_index.row);
    skip_whitespace(&state.lines, &mut start_index);
    let start_char_class = CharacterClass::from(state.lines.get(start_index));

    for (next_char, index) in state.lines.iter().from(start_index) {
        // Break loop if characters don't belong to the same class
        if CharacterClass::from(next_char) != start_char_class {
            break;
        }
        state.cursor = index;

        // Break loop if it reaches the end of the line
        if state.lines.is_last_col(index) {
            break;
        }
    }
}

/// Move one word forward. Breaks on the first character that is not of
/// the same class as the initial character or breaks on line starts.
/// Skips whitespaces if necessary.
#[derive(Clone, Debug, Copy)]
pub struct MoveWordBackward(pub usize);

impl Execute for MoveWordBackward {
    fn execute(&mut self, state: &mut EditorState) {
        if state.lines.is_empty() {
            return;
        }

        let max_col = max_col(&state.lines, &state.cursor, state.mode);
        if state.cursor.col > max_col {
            state.cursor.col = max_col;
        }

        for _ in 0..self.0 {
            move_word_backward(state);
        }

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

fn move_word_backward(state: &mut EditorState) {
    let mut start_index = state.cursor;
    if start_index.row == 0 && start_index.col == 0 {
        return;
    }

    if start_index.col == 0 {
        state.cursor.row = start_index.row.saturating_sub(1);
        state.cursor.col = state.lines.last_col_index(state.cursor.row);
        return;
    }

    start_index.col = start_index.col.saturating_sub(1);
    skip_whitespace_rev(&state.lines, &mut start_index);
    let start_char_class = CharacterClass::from(state.lines.get(start_index));

    for (next_char, i) in state.lines.iter().from(start_index).rev() {
        // Break loop if it reaches the start of the line
        if i.col == 0 {
            start_index = i;
            break;
        }
        // Break loop if characters don't belong to the same class
        if CharacterClass::from(next_char) != start_char_class {
            break;
        }
        start_index = i;
    }

    state.cursor = start_index;
}

// Move the cursor to the start of the line.
#[derive(Clone, Debug, Copy)]
pub struct MoveToStartOfLine();

impl Execute for MoveToStartOfLine {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.col = 0;

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}
// move to the first non-whitespace character in the line.
#[derive(Clone, Debug, Copy)]
pub struct MoveToFirst();

impl Execute for MoveToFirst {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.col = 0;
        skip_whitespace(&state.lines, &mut state.cursor);

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

// Move the cursor to the end of the line.
#[derive(Clone, Debug, Copy)]
pub struct MoveToEndOfLine();

impl Execute for MoveToEndOfLine {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.col = max_col(&state.lines, &state.cursor, state.mode);

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

// Move the cursor to the start of the buffer.
#[derive(Clone, Debug, Copy)]
pub struct MoveToFirstRow();

impl Execute for MoveToFirstRow {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.row = 0;

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

// Move the cursor to the end of the buffer.
#[derive(Clone, Debug, Copy)]
pub struct MoveToLastRow();

impl Execute for MoveToLastRow {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.row = state.lines.len().saturating_sub(1);

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

// Move the cursor to the closing bracket.
#[derive(Clone, Debug, Copy)]
pub struct MoveToMatchinBracket();

impl Execute for MoveToMatchinBracket {
    fn execute(&mut self, state: &mut EditorState) {
        let max_col = max_col_normal(&state.lines, &state.cursor);
        let index = Index2::new(state.cursor.row, state.cursor.col.min(max_col));
        if let Some(index) = find_matching_bracket(&state.lines, index) {
            state.cursor = index;
            if state.mode == EditorMode::Visual {
                set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
            }
        };
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveHalfPageDown();

impl Execute for MoveHalfPageDown {
    fn execute(&mut self, state: &mut EditorState) {
        let jump_rows = state.view.num_rows / 2;
        state.cursor.row = min(state.cursor.row + jump_rows, state.lines.last_row_index());

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveHalfPageUp();

impl Execute for MoveHalfPageUp {
    fn execute(&mut self, state: &mut EditorState) {
        let jump_rows = state.view.num_rows / 2;
        state.cursor.row = state.cursor.row.saturating_sub(jump_rows);

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MovePageDown();

impl Execute for MovePageDown {
    fn execute(&mut self, state: &mut EditorState) {
        let jump_rows = state.view.num_rows;
        let max_viewport_y = state.lines.len().saturating_sub(jump_rows);

        // Scroll viewport down
        state.view.viewport.y = min(state.view.viewport.y + jump_rows, max_viewport_y);

        // Cursor at top of viewport
        state.cursor.row = state.view.viewport.y;

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MovePageUp();

impl Execute for MovePageUp {
    fn execute(&mut self, state: &mut EditorState) {
        let jump_rows = state.view.num_rows;

        // Scroll viewport up
        state.view.viewport.y = state.view.viewport.y.saturating_sub(jump_rows);

        // Cursor at bottom of viewport
        let last_visible_row = state.view.viewport.y + jump_rows.saturating_sub(1);
        state.cursor.row = min(last_visible_row, state.lines.last_row_index());

        if state.mode == EditorMode::Visual {
            set_selection_with_lines(&mut state.selection, state.cursor, &state.lines);
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub(crate) enum CharacterClass {
    Unknown,
    Alphanumeric,
    Punctuation,
    Whitespace,
}

impl From<&char> for CharacterClass {
    fn from(value: &char) -> Self {
        if value.is_ascii_alphanumeric() {
            return Self::Alphanumeric;
        }
        if value.is_ascii_punctuation() {
            return Self::Punctuation;
        }
        if value.is_ascii_whitespace() {
            return Self::Whitespace;
        }
        Self::Unknown
    }
}

impl From<Option<&char>> for CharacterClass {
    fn from(value: Option<&char>) -> Self {
        value.map_or(CharacterClass::Unknown, Self::from)
    }
}

impl PartialEq for CharacterClass {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CharacterClass::Unknown, _) | (_, CharacterClass::Unknown) => false,
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Index2, Lines};

    use super::*;
    fn test_state() -> EditorState {
        EditorState::new(Lines::from("Hello World!\n\n123."))
    }

    #[test]
    fn test_move_forward() {
        let mut state = test_state();

        MoveForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));

        MoveForward(10).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));

        MoveForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));
    }

    #[test]
    fn test_move_backward() {
        let mut state = test_state();
        state.cursor = Index2::new(0, 11);

        MoveBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));

        MoveBackward(10).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));

        MoveBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
    }

    #[test]
    fn test_move_down() {
        let mut state = test_state();
        state.cursor = Index2::new(0, 6);

        MoveDown(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 6));

        MoveDown(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 6));

        MoveDown(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 6));
    }

    #[test]
    fn test_move_up() {
        let mut state = test_state();
        state.cursor = Index2::new(2, 2);

        MoveUp(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 2));

        MoveUp(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 2));

        MoveUp(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 2));
    }

    #[test]
    fn test_move_word_forward() {
        let mut state = test_state();

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 6));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 0));

        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 3));
    }

    #[test]
    fn test_move_word_forward_with_punctuation() {
        let mut state = EditorState::new(Lines::from("forward (w)"));

        // Start at 'f', move forward through "forward" and skip space to land on '('
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 8));

        // At '(', move through '(' and skip space (none) to land on 'w'
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 9));

        // At 'w', move through 'w' to land on ')'
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));
    }

    #[test]
    fn test_move_word_forward_punctuation_detailed() {
        // Test the exact case from the bug report
        let mut state = EditorState::new(Lines::from("forward (w)"));

        // Start at 'f' (position 0)
        assert_eq!(state.cursor, Index2::new(0, 0));

        // First move: from 'f' through "forward" to '('
        MoveWordForward(1).execute(&mut state);
        assert_eq!(
            state.cursor,
            Index2::new(0, 8),
            "Should move from 'f' to '(' after skipping whitespace"
        );

        // Second move: from '(' to 'w' (this was the bug - it went to ')' instead)
        MoveWordForward(1).execute(&mut state);
        assert_eq!(
            state.cursor,
            Index2::new(0, 9),
            "Should move from '(' to 'w', not to ')'"
        );

        // Third move: from 'w' to ')'
        MoveWordForward(1).execute(&mut state);
        assert_eq!(
            state.cursor,
            Index2::new(0, 10),
            "Should move from 'w' to ')'"
        );

        // Test with multiple punctuation characters
        let mut state = EditorState::new(Lines::from("hello()world"));

        // From 'h' to '('
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 5));

        // From '(' through ')' to 'w' (consecutive punctuation treated as one word)
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 7));

        // From 'w' through "world" to end
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));
    }

    #[test]
    fn test_move_word_forward_single_line() {
        // "Hello World" - should land on 'W' first, then on 'd' (end of line)
        let mut state = EditorState::new(Lines::from("Hello World"));

        // Start at 'H', move to 'W'
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 6));

        // From 'W', move to end of "World" (position 10, the 'd')
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));

        // From 'd', should stay at 'd' (already at end)
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));
    }

    #[test]
    fn test_move_word_forward_single_line_insert_mode() {
        // In insert mode, cursor should land AFTER 'd' (position 11)
        let mut state = EditorState::new(Lines::from("Hello World"));
        state.mode = EditorMode::Insert;

        // Start at 'H', move to 'W'
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 6));

        // From 'W', move to position after "World" (position 11, after 'd')
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));
    }

    #[test]
    fn test_move_word_forward_out_of_bounds() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 99);
        MoveWordForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
    }

    #[test]
    fn test_move_page_down() {
        // After page down, viewport scrolls and cursor is at top of new viewport (screen row 0)
        let mut state = EditorState::new(Lines::from(
            "Line 0\nLine 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9",
        ));
        state.view.num_rows = 3; // Viewport shows 3 lines

        // Start at line 0, viewport at 0
        assert_eq!(state.cursor.row, 0);
        assert_eq!(state.view.viewport.y, 0);

        // Page down: viewport scrolls to line 3, cursor at top of viewport (line 3)
        MovePageDown().execute(&mut state);
        assert_eq!(state.view.viewport.y, 3);
        assert_eq!(state.cursor.row, 3); // cursor at screen row 0

        // Page down: viewport scrolls to line 6, cursor at top of viewport (line 6)
        MovePageDown().execute(&mut state);
        assert_eq!(state.view.viewport.y, 6);
        assert_eq!(state.cursor.row, 6); // cursor at screen row 0

        // Page down: viewport can only go to line 7 (10 lines - 3 visible = max 7)
        // cursor at top of viewport (line 7)
        MovePageDown().execute(&mut state);
        assert_eq!(state.view.viewport.y, 7);
        assert_eq!(state.cursor.row, 7); // cursor at screen row 0
    }

    #[test]
    fn test_move_page_up() {
        // After page up, viewport scrolls and cursor is at bottom of new viewport
        let mut state = EditorState::new(Lines::from(
            "Line 0\nLine 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9",
        ));
        state.view.num_rows = 3; // Viewport shows 3 lines
        state.cursor.row = 9; // Start at last line
        state.view.viewport.y = 7; // Viewport showing lines 7-9

        // Page up: viewport scrolls to line 4, cursor at bottom of viewport (line 6)
        MovePageUp().execute(&mut state);
        assert_eq!(state.view.viewport.y, 4);
        assert_eq!(state.cursor.row, 6); // cursor at screen row 2 (bottom)

        // Page up: viewport scrolls to line 1, cursor at bottom of viewport (line 3)
        MovePageUp().execute(&mut state);
        assert_eq!(state.view.viewport.y, 1);
        assert_eq!(state.cursor.row, 3); // cursor at screen row 2 (bottom)

        // Page up: viewport scrolls to line 0, cursor at bottom of viewport (line 2)
        MovePageUp().execute(&mut state);
        assert_eq!(state.view.viewport.y, 0);
        assert_eq!(state.cursor.row, 2); // cursor at screen row 2 (bottom)
    }

    #[test]
    fn test_move_word_forward_to_end_of_word() {
        let mut state = test_state();

        MoveWordForwardToEndOfWord(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));

        MoveWordForwardToEndOfWord(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));

        MoveWordForwardToEndOfWord(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));

        MoveWordForwardToEndOfWord(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 2));

        MoveWordForwardToEndOfWord(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 3));

        MoveWordForwardToEndOfWord(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 3));
    }

    #[test]
    fn test_move_word_backward() {
        let mut state = test_state();
        state.cursor = Index2::new(2, 3);

        MoveWordBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 0));

        MoveWordBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));

        MoveWordBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));

        MoveWordBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 6));

        MoveWordBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));

        MoveWordBackward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
    }

    #[test]
    fn test_move_to_start() {
        let mut state = test_state();
        state.cursor = Index2::new(0, 2);

        MoveToStartOfLine().execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
    }

    #[test]
    fn test_move_to_end() {
        let mut state = test_state();
        state.cursor = Index2::new(0, 2);

        MoveToEndOfLine().execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));
    }

    #[test]
    fn test_move_to_first() {
        let mut state = EditorState::new(Lines::from(" Hello"));
        state.cursor = Index2::new(0, 3);

        MoveToFirst().execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));
    }
}

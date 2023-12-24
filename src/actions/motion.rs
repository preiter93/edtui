use super::Execute;
use crate::{
    helper::{clamp_column, max_col, max_row, set_selection, skip_whitespace, skip_whitespace_rev},
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
            set_selection(&mut state.selection, state.cursor);
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
            state.cursor.col -= 1;
        }
        if state.mode == EditorMode::Visual {
            set_selection(&mut state.selection, state.cursor);
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
            state.cursor.row -= 1;
            clamp_column(state);
        }
        if state.mode == EditorMode::Visual {
            set_selection(&mut state.selection, state.cursor);
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct MoveDown(pub usize);

impl Execute for MoveDown {
    fn execute(&mut self, state: &mut EditorState) {
        for _ in 0..self.0 {
            if state.cursor.row >= max_row(state) {
                break;
            }
            state.cursor.row += 1;
            clamp_column(state);
        }
        if state.mode == EditorMode::Visual {
            set_selection(&mut state.selection, state.cursor);
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
        fn move_word(state: &mut EditorState) {
            let mut index = state.cursor;
            let lines = &state.lines;
            let first_char = state.lines.get(index);
            let mut iter = state.lines.iter().from(index);
            iter.next();
            for (val, i) in iter {
                index = i;
                // Break loop if it reaches the end of the line
                if state.cursor.col >= max_col(&state.lines, &state.cursor, state.mode) {
                    break;
                }
                // Break loop if characters don't belong to the same class
                if !is_same_word_class(val, first_char) {
                    break;
                }
            }
            // Skip whitespaces moving to the right.
            skip_whitespace(lines, &mut index);

            state.cursor = index;
        }

        for _ in 0..self.0 {
            move_word(state);
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
        fn move_word(state: &mut EditorState) {
            let mut index = state.cursor;
            let lines = &state.lines;
            if index.col == 0 {
                index.row = index.row.saturating_sub(1);
                index.col = lines.len_col(index.row).saturating_sub(1);
                state.cursor = index;
                return;
            }
            index.col = index.col.saturating_sub(1);

            // Skip whitespaces to the left
            skip_whitespace_rev(lines, &mut index);

            let first_char = lines.get(index);
            for (val, i) in lines.iter().from(index).rev() {
                // Break loop if it reaches the start of the line
                if i.col == 0 {
                    index = i;
                    break;
                }
                // Break loop if characters don't belong to the same class
                if !is_same_word_class(val, first_char) {
                    break;
                }
                index = i;
            }
            state.cursor = index;
        }

        for _ in 0..self.0 {
            move_word(state);
        }
    }
}

/// Whether two characters are considered of the same class.
fn is_same_word_class(a: Option<&char>, b: Option<&char>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => {
            a.is_ascii_alphanumeric() && b.is_ascii_alphanumeric()
                || (a.is_ascii_punctuation() && b.is_ascii_punctuation())
                || (a.is_ascii_whitespace() && b.is_ascii_whitespace())
        }
        _ => false,
    }
}

// Move the cursor to the start of the line.
#[derive(Clone, Debug, Copy)]
pub struct MoveToStart();

impl Execute for MoveToStart {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.col = 0;
    }
}
// move to the first non-whitespace character in the line.
#[derive(Clone, Debug, Copy)]
pub struct MoveToFirst();

impl Execute for MoveToFirst {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.col = 0;
    }
}

// Move the cursor to the end of the line.
#[derive(Clone, Debug, Copy)]
pub struct MoveToEnd();

impl Execute for MoveToEnd {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.col = max_col(&state.lines, &state.cursor, state.mode);
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
        assert_eq!(state.cursor, Index2::new(1, 0));

        MoveDown(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 0));

        MoveDown(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 0));
    }

    #[test]
    fn test_move_up() {
        let mut state = test_state();
        state.cursor = Index2::new(2, 2);

        MoveUp(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));

        MoveUp(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));

        MoveUp(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
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
    }

    #[test]
    fn test_move_to_start() {
        let mut state = test_state();
        state.cursor = Index2::new(0, 2);

        MoveToStart().execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
    }

    #[test]
    fn test_move_to_end() {
        let mut state = test_state();
        state.cursor = Index2::new(0, 2);

        MoveToEnd().execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 11));
    }
}

use jagged::index::RowIndex;

use super::{Execute, SwitchMode};
use crate::{
    helper::{insert_char, line_break},
    EditorMode, EditorState,
};

/// Inserts a single character at the current cursor position
#[derive(Clone, Debug, Copy)]
pub struct InsertChar(pub char);

impl Execute for InsertChar {
    fn execute(&mut self, state: &mut EditorState) {
        insert_char(&mut state.lines, &mut state.cursor, self.0, false);
    }
}

/// Inserts a newline at the current cursor position
#[derive(Clone, Debug, Copy)]
pub struct LineBreak(pub usize);

impl Execute for LineBreak {
    fn execute(&mut self, state: &mut EditorState) {
        if state.lines.is_empty() {
            state.lines.push(Vec::new());
        }
        for _ in 0..self.0 {
            line_break(&mut state.lines, &mut state.cursor);
        }
    }
}

/// Appends a newline below the current cursor position
/// and switches into insert mode.
#[derive(Clone, Debug, Copy)]
pub struct AppendNewline(pub usize);

impl Execute for AppendNewline {
    fn execute(&mut self, state: &mut EditorState) {
        SwitchMode(EditorMode::Insert).execute(state);
        state.cursor.col = 0;
        for _ in 0..self.0 {
            if !state.lines.is_empty() {
                state.cursor.row += 1;
            }
            state.lines.insert(RowIndex::new(state.cursor.row), vec![]);
        }
    }
}

/// Appends a newline at the current cursor position
/// and switches into insert mode.
#[derive(Clone, Debug, Copy)]
pub struct InsertNewline(pub usize);

impl Execute for InsertNewline {
    fn execute(&mut self, state: &mut EditorState) {
        SwitchMode(EditorMode::Insert).execute(state);
        state.cursor.col = 0;
        for _ in 0..self.0 {
            state.lines.insert(RowIndex::new(state.cursor.row), vec![]);
        }
    }
}

/// Pushes a line to the back of the buffer.
/// Does not affect the cursor position.
#[derive(Clone, Debug, Copy)]
pub struct PushLine<'a>(pub &'a str);

impl Execute for PushLine<'_> {
    fn execute(&mut self, state: &mut EditorState) {
        let chars: Vec<char> = self.0.chars().collect();
        state.lines.push(chars);
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
    fn test_insert_char() {
        let mut state = test_state();

        InsertChar('!').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.lines, Lines::from("!Hello World!\n\n123."));

        state.cursor = Index2::new(0, 13);
        InsertChar('!').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 14));
        assert_eq!(state.lines, Lines::from("!Hello World!!\n\n123."));
    }

    #[test]
    fn test_insert_char_into_empty_buffer() {
        let mut state = EditorState::new(Lines::from("\n"));
        state.cursor.row = 1;

        InsertChar('a').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 1));
        assert_eq!(state.lines, Lines::from("\na"));
    }

    #[test]
    fn test_insert_char_out_of_bounds() {
        let mut state = EditorState::new(Lines::from("\nb"));
        state.cursor = Index2::new(0, 1);

        InsertChar('a').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.lines, Lines::from("a\nb"));
    }

    #[test]
    fn test_line_break() {
        let mut state = test_state();

        LineBreak(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
        assert_eq!(state.lines, Lines::from("\nHello World!\n\n123."));

        state.cursor = Index2::new(1, 5);
        LineBreak(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 0));
        assert_eq!(state.lines, Lines::from("\nHello\n World!\n\n123."));
    }

    #[test]
    fn test_line_break_col_out_of_bounds() {
        let mut state = test_state();
        state.cursor.col = 99;

        LineBreak(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
        assert_eq!(state.lines, Lines::from("Hello World!\n\n\n123."));

        state.cursor.col = 99;
        state.cursor.row = 4;
        LineBreak(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(5, 0));
        assert_eq!(state.lines, Lines::from("Hello World!\n\n\n123.\n"));
    }

    #[test]
    fn test_append_newline() {
        let mut state = test_state();

        AppendNewline(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
        assert_eq!(state.lines, Lines::from("Hello World!\n\n\n123."));
    }

    #[test]
    fn test_insert_newline() {
        let mut state = test_state();

        InsertNewline(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from("\nHello World!\n\n123."));

        state.cursor = Index2::new(2, 1);
        InsertNewline(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(2, 0));
        assert_eq!(state.lines, Lines::from("\nHello World!\n\n\n123."));
    }

    #[test]
    fn test_push_line() {
        let mut state = test_state();

        PushLine("456.").execute(&mut state);
        assert_eq!(state.lines, Lines::from("Hello World!\n\n123.\n456."));
    }
}

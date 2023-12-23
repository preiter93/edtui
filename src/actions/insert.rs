use jagged::index::RowIndex;

use super::{Execute, SwitchMode};
use crate::{EditorMode, EditorState};

/// Inserts a single character at the current cursor position
#[derive(Clone, Debug, Copy)]
pub struct InsertChar(pub char);

impl Execute for InsertChar {
    fn execute(&mut self, state: &mut EditorState) {
        let ch = self.0;
        if state.lines.is_empty() {
            state.lines.push(Vec::new());
        }
        if ch == '\n' {
            LineBreak(1).execute(state);
        } else {
            state.lines.insert(state.cursor.as_index(), ch);
            state.cursor.column += 1;
        }
    }
}

/// Inserts a text at the current cursor position
#[derive(Clone, Debug)]
pub struct InsertString(pub String);

impl Execute for InsertString {
    fn execute(&mut self, state: &mut EditorState) {
        for ch in self.0.chars() {
            InsertChar(ch).execute(state);
        }
    }
}

/// Inserts a newline at the current cursor position
#[derive(Clone, Debug, Copy)]
pub struct LineBreak(pub usize);

impl Execute for LineBreak {
    fn execute(&mut self, state: &mut EditorState) {
        for _ in 0..self.0 {
            if state.cursor.column == 0 {
                state.lines.insert(RowIndex::new(state.cursor.line), vec![]);
            } else {
                let split_at = state.cursor.as_index();
                let mut rest = state.lines.split_off(split_at);
                state.lines.append(&mut rest);
            }
            state.cursor.line += 1;
            state.cursor.column = 0;
        }
    }
}
/// Appends a newline below the current cursor position
/// and switches into insert mode.
#[derive(Clone, Debug, Copy)]
pub struct AppendNewline(pub usize);

impl Execute for AppendNewline {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.column = 0;
        for _ in 0..self.0 {
            state.cursor.line += 1;
            state.lines.insert(RowIndex::new(state.cursor.line), vec![]);
        }
        SwitchMode(EditorMode::Insert).execute(state);
    }
}

/// Appends a newline at the current cursor position
/// and switches into insert mode.
#[derive(Clone, Debug, Copy)]
pub struct InsertNewline(pub usize);

impl Execute for InsertNewline {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.column = 0;
        for _ in 0..self.0 {
            state.lines.insert(RowIndex::new(state.cursor.line), vec![]);
        }
        SwitchMode(EditorMode::Insert).execute(state);
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
    use crate::{state::position::Position, Lines};

    use super::*;
    fn test_state() -> EditorState {
        EditorState::new(Lines::from("Hello World!\n\n123."))
    }

    #[test]
    fn test_insert_char() {
        let mut state = test_state();

        InsertChar('!').execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 1));
        assert_eq!(state.lines, Lines::from("!Hello World!\n\n123."));

        state.set_cursor_position(0, 13);
        InsertChar('!').execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 14));
        assert_eq!(state.lines, Lines::from("!Hello World!!\n\n123."));
    }

    #[test]
    fn test_insert_str() {
        let mut state = test_state();

        state.set_cursor_position(0, 5);
        InsertString(String::from(",\nx")).execute(&mut state);
        assert_eq!(state.cursor, Position::new(1, 1));
        assert_eq!(state.lines, Lines::from("Hello,\nx World!\n\n123."));
    }

    #[test]
    fn test_linebreak() {
        let mut state = test_state();

        LineBreak(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(1, 0));
        assert_eq!(state.lines, Lines::from("\nHello World!\n\n123."));

        state.set_cursor_position(1, 5);
        LineBreak(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(2, 0));
        assert_eq!(state.lines, Lines::from("\nHello\n World!\n\n123."));
    }

    #[test]
    fn test_append_newline() {
        let mut state = test_state();

        AppendNewline(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(1, 0));
        assert_eq!(state.lines, Lines::from("Hello World!\n\n\n123."));
    }

    #[test]
    fn test_insert_newline() {
        let mut state = test_state();

        InsertNewline(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(0, 0));
        assert_eq!(state.lines, Lines::from("\nHello World!\n\n123."));

        state.set_cursor_position(2, 1);
        InsertNewline(1).execute(&mut state);
        assert_eq!(state.cursor, Position::new(2, 0));
        assert_eq!(state.lines, Lines::from("\nHello World!\n\n\n123."));
    }

    #[test]
    fn test_push_line() {
        let mut state = test_state();

        PushLine("456.").execute(&mut state);
        assert_eq!(state.lines, Lines::from("Hello World!\n\n123.\n456."));
    }
}

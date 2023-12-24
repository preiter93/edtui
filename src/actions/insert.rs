use jagged::index::RowIndex;

use super::Execute;
use crate::{EditorMode, EditorState, Index2, Lines};

/// Inserts a single character at the current cursor position
#[derive(Clone, Debug, Copy)]
pub struct InsertChar(pub char);

impl Execute for InsertChar {
    fn execute(&mut self, state: &mut EditorState) {
        insert_char(&mut state.lines, &mut state.cursor, self.0);
    }
}

fn insert_char(lines: &mut Lines, index: &mut Index2, ch: char) {
    if lines.is_empty() {
        lines.push(Vec::new());
    }
    if ch == '\n' {
        line_break(lines, index);
    } else {
        lines.insert(*index, ch);
        index.col += 1;
    }
}

/// Inserts a text at the current cursor position
#[derive(Clone, Debug)]
pub struct InsertString(pub String);

impl Execute for InsertString {
    fn execute(&mut self, state: &mut EditorState) {
        for ch in self.0.chars() {
            insert_char(&mut state.lines, &mut state.cursor, ch);
        }
    }
}

/// Inserts a newline at the current cursor position
#[derive(Clone, Debug, Copy)]
pub struct LineBreak(pub usize);

impl Execute for LineBreak {
    fn execute(&mut self, state: &mut EditorState) {
        for _ in 0..self.0 {
            line_break(&mut state.lines, &mut state.cursor);
        }
    }
}

fn line_break(lines: &mut Lines, index: &mut Index2) {
    if index.col == 0 {
        lines.insert(RowIndex::new(index.row), vec![]);
    } else {
        let mut rest = lines.split_off(*index);
        lines.append(&mut rest);
    }
    index.row += 1;
    index.col = 0;
}

/// Appends a newline below the current cursor position
/// and switches into insert mode.
#[derive(Clone, Debug, Copy)]
pub struct AppendNewline(pub usize);

impl Execute for AppendNewline {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.col = 0;
        for _ in 0..self.0 {
            state.cursor.row += 1;
            state.lines.insert(RowIndex::new(state.cursor.row), vec![]);
        }
        state.mode = EditorMode::Insert;
    }
}

/// Appends a newline at the current cursor position
/// and switches into insert mode.
#[derive(Clone, Debug, Copy)]
pub struct InsertNewline(pub usize);

impl Execute for InsertNewline {
    fn execute(&mut self, state: &mut EditorState) {
        state.cursor.col = 0;
        for _ in 0..self.0 {
            state.lines.insert(RowIndex::new(state.cursor.row), vec![]);
        }
        state.mode = EditorMode::Insert;
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
    use crate::Lines;

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
    fn test_insert_str() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 5);
        InsertString(String::from(",\nx")).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 1));
        assert_eq!(state.lines, Lines::from("Hello,\nx World!\n\n123."));
    }

    #[test]
    fn test_linebreak() {
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

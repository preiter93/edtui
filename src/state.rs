pub mod mode;
pub mod position;
pub mod selection;
mod undo;
mod view;

use self::view::ViewState;
use self::{mode::EditorMode, position::Position, selection::Selection, undo::Stack};
use crate::lines::{Line, Lines};

#[derive(Clone)]
pub struct EditorState {
    pub lines: Lines,
    pub cursor: Position,
    pub mode: EditorMode,
    pub selection: Option<Selection>,
    pub(crate) view: ViewState,
    undo: Stack,
    redo: Stack,
}

impl Default for EditorState {
    fn default() -> Self {
        EditorState::new()
    }
}

impl EditorState {
    /// Create a new empty Buffer.
    pub fn new() -> EditorState {
        EditorState {
            lines: Lines::new(),
            cursor: Position::new(0, 0),
            mode: EditorMode::Normal,
            selection: None,
            view: ViewState::default(),
            undo: Stack::new(),
            redo: Stack::new(),
        }
    }

    /// Switch to normal
    pub fn normal_mode(&mut self) {
        self.mode = EditorMode::Normal;
        self.selection = None;
    }

    /// Switch to insert mode
    pub fn insert_mode(&mut self) {
        self.mode = EditorMode::Insert;
    }

    /// Switch to visual mode
    pub fn visual_mode(&mut self) {
        self.mode = EditorMode::Visual;
        self.set_selection(self.cursor.clone(), self.cursor.clone());
    }

    /// Switch to insert mode and move one character forward
    pub fn append_mode(&mut self) {
        self.mode = EditorMode::Insert;
        if self.cursor.column < self.column_len() {
            self.cursor.column += 1;
        }
    }

    /// Returns the text in the buffer as a vector of lines
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    /// Clears the buffer
    pub fn clear(&mut self) {
        self.lines.clear();
        self.cursor = Position::new(0, 0);
        self.selection = None;
    }

    /// Appends a line to the back of the buffer.
    pub fn push<T: Into<Line>>(&mut self, line: T) {
        self.lines.push(line.into())
    }

    /// Inserts a line to the back of the buffer.
    pub fn insert<T: Into<Line>>(&mut self, index: usize, line: T) {
        self.lines.insert(index, line.into())
    }

    /// Inserts a single character at the current cursor position
    pub fn insert_char(&mut self, ch: char) {
        if self.lines().is_empty() {
            self.lines.push("");
        }
        if ch == '\n' {
            self.insert_newline()
        } else {
            let line = &mut self.lines[self.cursor.line];
            line.insert(self.cursor.column, ch);
            self.cursor.column += 1;
        }
    }

    /// Inserts a text at the current cursor position
    pub fn insert_string(&mut self, string: &str) {
        for ch in string.chars() {
            self.insert_char(ch);
        }
    }

    /// Inserts a newline at the current cursor position
    pub fn insert_newline(&mut self) {
        self.lines.insert_newline(&self.cursor);
        self.cursor.line += 1;
        self.cursor.column = 0;
    }

    /// Create a new empty line below and switch to insert mode
    pub fn new_line_below_and_insert_mode(&mut self) {
        self.insert_newline_below();
        self.insert_mode();
    }

    /// Create a new empty line above and switch to insert mode
    pub fn new_line_above_and_insert_mode(&mut self) {
        self.insert_newline_above();
        self.insert_mode();
    }

    /// Inserts a newline below the current cursor position
    fn insert_newline_below(&mut self) {
        let index = self.cursor.line + 1;
        self.insert(index, "");
        self.cursor.line += 1;
        self.cursor.column = 0;
    }

    /// Inserts a newline above the current cursor position
    fn insert_newline_above(&mut self) {
        let index = self.cursor.line;
        self.insert(index, "");
        self.cursor.column = 0;
    }

    /// Set the cursor position to the specified line and column
    pub fn set_cursor_position(&mut self, line: usize, column: usize) {
        self.cursor.line = line;
        self.cursor.column = column;
    }

    /// Set the editing mode to the specified mode
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    /// Set the selection to the specified start and end positions
    pub fn set_selection(&mut self, start: Position, end: Position) {
        self.selection = Some(Selection { start, end });
    }

    /// Set the selection end positions
    pub fn set_selection_end(&mut self, end: Position) {
        if let Some(start) = self.selection.as_ref().map(|x| x.start.clone()) {
            self.selection = Some(Selection { start, end });
        }
    }

    /// Clear the selection, setting it to None
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Move the cursor left by one column, wrapping to the previous line if needed
    pub fn move_cursor_left(&mut self) {
        if self.cursor.column > 0 {
            self.cursor.column -= 1;
        } else if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.column = self.column_len();
        }

        if self.mode == EditorMode::Visual {
            let cursor = self.cursor.clone();
            self.set_selection_end(cursor);
        }
    }

    /// Move the cursor right by one column, wrapping to the next line if needed
    pub fn move_cursor_right(&mut self) {
        if self.cursor.column < self.column_len() {
            self.cursor.column += 1;
        } else if self.cursor.line + 1 < self.lines.len() {
            self.cursor.line += 1;
            self.cursor.column = 0;
        }

        if self.mode == EditorMode::Visual {
            let cursor = self.cursor.clone();
            self.set_selection_end(cursor);
        }
    }

    /// Move the cursor up by one line
    pub fn move_cursor_up(&mut self) {
        if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.column = self.cursor.column.min(self.column_len());
        }

        if self.mode == EditorMode::Visual {
            let cursor = self.cursor.clone();
            self.set_selection_end(cursor);
        }
    }

    /// Move the cursor down by one line
    pub fn move_cursor_down(&mut self) {
        if !self.lines().is_empty() && self.cursor.line < self.lines.len() - 1 {
            self.cursor.line += 1;
            self.cursor.column = self.cursor.column.min(self.column_len());
        }

        if self.mode == EditorMode::Visual {
            let cursor = self.cursor.clone();
            self.set_selection_end(cursor);
        }
    }

    /// Move the cursor to the beginning of the line
    pub fn move_cursor_to_beginning_of_line(&mut self) {
        self.cursor.column = 0;

        if self.mode == EditorMode::Visual {
            let cursor = self.cursor.clone();
            self.set_selection_end(cursor);
        }
    }

    /// Move the cursor to the end of the line
    pub fn move_cursor_to_end_of_line(&mut self) {
        self.cursor.column = self.column_len().saturating_sub(1);

        if self.mode == EditorMode::Visual {
            let cursor = self.cursor.clone();
            self.set_selection_end(cursor);
        }
    }

    /// Move the cursor right by one word, wrapping to the next line if needed
    pub fn move_cursor_one_word_right(&mut self) {
        let first_char = self.lines.char_at(&self.cursor);
        self.cursor = self.lines.next_word(&self.cursor);

        // Wrap to new line if necessary
        if let (Some(a), Some(b)) = (first_char, self.lines.char_at(&self.cursor)) {
            if Self::is_same_word_class(*a, *b) {
                self.cursor = self.lines.wrap_forward(&self.cursor);
            }
        }
    }

    ///  Move the cursor left by one word, wrapping to the next line if needed
    pub fn move_cursor_one_word_left(&mut self) {
        self.cursor = self.lines.prev_word(&self.cursor);

        // TODO: Test wrapping
    }

    /// Deletes a single character at the current cursor position. Does
    /// not move the cursor position unless it is at the end of the line.
    pub fn remove_char(&mut self) {
        if self.column_len() == 0 {
            return;
        }
        self.lines[self.cursor.line].remove(self.cursor.column);
        self.cursor.column = self.cursor.column.min(self.column_len().saturating_sub(1));
    }

    /// Deletes a single character at the current cursor position. Moves the cursor
    /// position by one and jumps to the previous line if necessary.
    pub fn delete_char(&mut self) {
        if self.cursor.column == 0 && self.cursor.line == 0 {
            return;
        }

        if self.cursor.column == 0 {
            self.move_cursor_left();
            let next_line_idx = self.cursor.line + 1;
            if let Some(next_line) = self.lines.get(next_line_idx).cloned() {
                self.lines[self.cursor.line].push_chars(&next_line);
                self.lines.remove(next_line_idx);
            }
        } else {
            self.move_cursor_left();
            self.lines[self.cursor.line].remove(self.cursor.column);
        }
    }

    /// Delete the current line.
    pub fn delete_line(&mut self) {
        if self.cursor.line < self.lines.len() {
            self.lines.remove(self.cursor.line);
            self.cursor.column = 0;
            self.cursor.line = self.cursor.line.min(self.lines.len().saturating_sub(1));
        }
    }

    /// Delete between a start and end position.
    fn delete_between(&mut self, start: &Position, end: &Position) {
        self.cursor = end.clone();
        self.move_cursor_right();
        self.delete_char();
        while &self.cursor != start {
            self.delete_char();
        }
    }

    /// Delete the current selection.
    pub fn delete_selection(&mut self) {
        if let Some(selection) = self.selection.take() {
            self.cursor = selection.end();
            self.move_cursor_right();
            self.delete_char();
            while self.cursor != selection.start() {
                self.delete_char();
            }
            // let _ = self.lines.drain(&selection.start, &selection.end);
        }
    }

    /// Selects and deletes text between specified delimiter characters.
    ///
    /// This function takes an input text and a list of delimiter characters as input.
    /// It searches for the first occurrence of a delimiter character in the text to
    /// define the start of the selection, and the next occurrence of any of the delimiter
    /// characters to define the end of the selection.
    pub fn select_between_delimiters(&mut self, delimiters: &[char]) {
        let cursor = self.cursor.clone();
        let mut start: Option<Position> = None;
        let mut end: Option<Position> = None;
        let mut prev = cursor.clone();
        for pos in self.lines.pos_iter().start(cursor.clone()) {
            if let Some(c) = self.lines.char_at(&pos) {
                if delimiters.contains(c) {
                    end = Some(prev);
                    break;
                }
            }
            prev = pos;
        }
        prev = cursor.clone();
        for pos in self.lines.pos_iter().start(cursor).rev() {
            if let Some(c) = self.lines.char_at(&pos) {
                if delimiters.contains(c) {
                    start = Some(prev);
                    break;
                }
            }
            prev = pos;
        }
        if let (Some(start), Some(end)) = (start, end) {
            self.selection = Some(Selection { start, end });
        }
    }

    /// Returns the text in between a selection.
    #[must_use]
    fn text_between_selection(&self) -> String {
        if let Some(selection) = &self.selection {
            let start = selection.start();
            let target = selection.end();
            let mut prev_line = start.line;
            return self
                .lines
                .pos_iter()
                .start(start)
                .take_until(|pos| pos == &target)
                .fold(String::new(), |mut value, pos| {
                    for _ in prev_line..pos.line {
                        value.push('\n');
                    }
                    if let Some(ch) = self.lines.char_at(&pos) {
                        value.push(*ch);
                    }
                    prev_line = pos.line;
                    value
                });
        }
        String::new()
    }

    /// Get the number of columns in the current line.
    pub fn column_len(&self) -> usize {
        self.column_len_at(self.cursor.line)
    }

    /// Get the number of columns of a line from the line index.
    pub fn column_len_at(&self, index: usize) -> usize {
        match self.lines.get(index) {
            Some(line) => line.len(),
            None => 0,
        }
    }

    /// Get the curently selected character.
    pub fn char(&self) -> Option<&char> {
        let line = &self.lines[self.cursor.line];
        line.iter().nth(self.cursor.column)
    }

    /// Wwhether the cursor is at the of the current line.
    pub fn is_cursor_at_end_of_line(&self) -> bool {
        self.column_len() <= self.cursor.column + 1
    }

    /// Wwhether the cursor is at the of the current line.
    pub fn is_cursor_at_start_of_line(&self) -> bool {
        self.cursor.column == 0
    }

    /// Wwhether the cursor is at the last line.
    pub fn is_cursor_at_last_line(&self) -> bool {
        self.lines.len() <= self.cursor.line + 1
    }

    /// Whether the cursor is at the end of the buffer.
    pub fn is_cursor_at_end(&self) -> bool {
        self.is_cursor_at_end_of_line() && self.is_cursor_at_last_line()
    }

    /// Whether the cursor is at the beginning of the buffer.
    pub fn is_cursor_at_start(&self) -> bool {
        self.cursor.line == 0 && self.cursor.column == 0
    }

    /// Whether two characters are considered of the same class.
    fn is_same_word_class(a: char, b: char) -> bool {
        a.is_ascii_alphanumeric() && b.is_ascii_alphanumeric()
            || (a.is_ascii_punctuation() && b.is_ascii_punctuation())
            || (a.is_ascii_whitespace() && b.is_ascii_whitespace())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_cursor_left() {
        let mut buffer = EditorState::new();
        buffer.set_cursor_position(0, 3);
        buffer.push("Line 1");

        buffer.move_cursor_left();
        assert_eq!(buffer.cursor.column, 2);

        buffer.move_cursor_left();
        assert_eq!(buffer.cursor.column, 1);

        buffer.move_cursor_left();
        assert_eq!(buffer.cursor.column, 0);

        buffer.move_cursor_left();
        assert_eq!(buffer.cursor.column, 0);
    }

    #[test]
    fn test_move_cursor_right() {
        let mut buffer = EditorState::new();
        buffer.set_cursor_position(0, 4);
        buffer.push("Line 1");

        buffer.move_cursor_right();
        assert_eq!(buffer.cursor.column, 5);

        buffer.move_cursor_right();
        assert_eq!(buffer.cursor.column, 6);

        buffer.move_cursor_right();
        assert_eq!(buffer.cursor.column, 6);
    }

    #[test]
    fn test_move_cursor_up() {
        let mut buffer = EditorState::new();
        buffer.set_cursor_position(2, 3);
        buffer.push("Line 1");
        buffer.push("Line 2");
        buffer.push("Line 3");

        buffer.move_cursor_up();
        assert_eq!(buffer.cursor.line, 1);

        buffer.move_cursor_up();
        assert_eq!(buffer.cursor.line, 0);

        buffer.move_cursor_up();
        assert_eq!(buffer.cursor.line, 0);
    }

    #[test]
    fn test_move_cursor_down() {
        let mut buffer = EditorState::new();
        buffer.set_cursor_position(1, 3);
        buffer.push("Line 1");
        buffer.push("Line 2");
        buffer.push("Line 3");

        buffer.move_cursor_down();
        assert_eq!(buffer.cursor.line, 2);

        buffer.move_cursor_down();
        assert_eq!(buffer.cursor.line, 2);
    }

    #[test]
    fn test_move_cursor_to_beginning_of_line() {
        let mut buffer = EditorState::new();
        buffer.set_cursor_position(1, 3);
        buffer.push("Line 1");
        buffer.push("Line 2");
        buffer.push("Line 3");

        buffer.move_cursor_to_beginning_of_line();
        assert_eq!(buffer.cursor.column, 0);
    }

    #[test]
    fn test_move_cursor_to_end_of_line() {
        let mut buffer = EditorState::new();
        buffer.set_cursor_position(1, 3);
        buffer.push("Line 1");
        buffer.push("Line 2");
        buffer.push("Line 3");

        buffer.move_cursor_to_end_of_line();
        assert_eq!(buffer.cursor.column, 5);
    }

    #[test]
    fn test_set_cursor_position() {
        let mut buffer = EditorState::new();
        buffer.push("Line 1");
        buffer.push("Line 2");
        buffer.push("Line 3");

        buffer.set_cursor_position(0, 3);
        assert_eq!(buffer.cursor.line, 0);
        assert_eq!(buffer.cursor.column, 3);

        buffer.set_cursor_position(2, 5);
        assert_eq!(buffer.cursor.line, 2);
        assert_eq!(buffer.cursor.column, 5);
    }

    #[test]
    fn test_move_one_word_right() {
        let mut buffer = EditorState::new();
        buffer.set_cursor_position(0, 0);
        buffer.push("Hello, world!");
        buffer.push(" Boom.");

        buffer.move_cursor_one_word_right();
        assert_eq!(buffer.cursor.column, 5);

        buffer.move_cursor_one_word_right();
        assert_eq!(buffer.cursor.column, 7);

        buffer.move_cursor_one_word_right();
        assert_eq!(buffer.cursor.column, 12);

        buffer.move_cursor_one_word_right();
        assert_eq!(buffer.cursor.line, 1);
        assert_eq!(buffer.cursor.column, 1);

        buffer.move_cursor_one_word_right();
        assert_eq!(buffer.cursor.line, 1);
        assert_eq!(buffer.cursor.column, 5);

        buffer.move_cursor_one_word_right();
        assert_eq!(buffer.cursor.line, 1);
        assert_eq!(buffer.cursor.column, 5);
    }

    #[test]
    fn test_move_one_word_left() {
        let mut buffer = EditorState::new();
        buffer.set_cursor_position(0, 18);
        buffer.push("Hello, world! Boom.");

        buffer.move_cursor_one_word_left();
        assert_eq!(buffer.cursor.column, 14);

        buffer.move_cursor_one_word_left();
        assert_eq!(buffer.cursor.column, 12);

        buffer.move_cursor_one_word_left();
        assert_eq!(buffer.cursor.column, 7);

        buffer.move_cursor_one_word_left();
        assert_eq!(buffer.cursor.column, 5);

        buffer.move_cursor_one_word_left();
        assert_eq!(buffer.cursor.column, 0);

        buffer.move_cursor_one_word_left();
        assert_eq!(buffer.cursor.column, 0);
    }

    #[test]
    fn test_remove_char() {
        let mut buffer = EditorState::new();
        let mut expect;
        buffer.set_cursor_position(0, 4);
        buffer.push("Line 1");

        buffer.remove_char();
        expect = vec![Line::from("Line1")];
        assert_eq!(buffer.lines.to_vec(), expect);
        assert_eq!(buffer.cursor.column, 4);

        buffer.remove_char();
        expect = vec![Line::from("Line")];
        assert_eq!(buffer.lines.to_vec(), expect);
        assert_eq!(buffer.cursor.column, 3);
    }

    #[test]
    fn test_delete_char() {
        let mut buffer = EditorState::new();
        let mut expect;
        buffer.push("Line 1");
        buffer.push("");
        buffer.push("Line 3");

        buffer.set_cursor_position(0, 2);
        buffer.delete_char();

        expect = vec![Line::from("Lne 1"), Line::from(""), Line::from("Line 3")];
        assert_eq!(buffer.lines().to_vec(), expect);
        assert_eq!(buffer.cursor, Position::new(0, 1));

        buffer.set_cursor_position(2, 2);
        for _ in 0..4 {
            buffer.delete_char();
        }

        expect = vec![Line::from("Lne 1ne 3")];
        assert_eq!(buffer.lines().to_vec(), expect);
        assert_eq!(buffer.cursor, Position::new(0, 5));

        buffer.set_cursor_position(0, 9);
        buffer.delete_char();

        expect = vec![Line::from("Lne 1ne ")];
        assert_eq!(buffer.lines().to_vec(), expect);
        assert_eq!(buffer.cursor, Position::new(0, 8));
    }

    #[test]
    fn test_delete_line() {
        let mut buffer = EditorState::new();
        buffer.set_cursor_position(1, 0);
        buffer.push("Line 1");
        buffer.push("Line 2");
        buffer.push("Line 3");
        let mut buffer_exp = EditorState::new();

        buffer.delete_line();
        buffer_exp.push("Line 1");
        buffer_exp.push("Line 3");
        assert_eq!(buffer.lines, buffer_exp.lines);
        assert_eq!(buffer.cursor.line, 1);
        assert_eq!(buffer.cursor.column, 0);

        buffer.delete_line();
        buffer_exp.lines.clear();
        buffer_exp.push("Line 1");
        assert_eq!(buffer.lines, buffer_exp.lines);
        assert_eq!(buffer.cursor.line, 0);
        assert_eq!(buffer.cursor.column, 0);

        buffer.delete_line();
        buffer_exp.lines.clear();
        assert_eq!(buffer.lines, buffer_exp.lines);
        assert_eq!(buffer.cursor.line, 0);
        assert_eq!(buffer.cursor.column, 0);

        buffer.delete_line();
        buffer_exp.lines.clear();
        assert_eq!(buffer.lines, buffer_exp.lines);
        assert_eq!(buffer.cursor.line, 0);
        assert_eq!(buffer.cursor.column, 0);
    }

    #[test]
    fn test_delete_selection() {
        let mut buffer = EditorState::new();
        buffer.push("Line 1");
        buffer.push("Line 2");
        buffer.selection = Some(Selection {
            start: Position { line: 0, column: 0 },
            end: Position { line: 1, column: 2 },
        });
        let mut buffer_exp = EditorState::new();
        buffer_exp.push("e 2");

        buffer.delete_selection();
        assert_eq!(buffer.lines, buffer_exp.lines);
    }

    #[test]
    fn test_is_cursor_at_end_of_line() {
        let mut buffer = EditorState::new();
        buffer.push("Line 1");
        buffer.set_cursor_position(0, 5);
        assert!(buffer.is_cursor_at_end_of_line());

        buffer.set_cursor_position(0, 2);
        assert!(!buffer.is_cursor_at_end_of_line());

        let empty_buffer = EditorState::new();
        assert!(empty_buffer.is_cursor_at_end_of_line());
    }

    #[test]
    fn test_is_cursor_at_last_line() {
        let mut buffer = EditorState::new();
        buffer.push("Line 1");
        buffer.push("Line 2");
        buffer.set_cursor_position(1, 5);
        assert!(buffer.is_cursor_at_last_line());

        buffer.set_cursor_position(0, 5);
        assert!(!buffer.is_cursor_at_last_line());

        let empty_buffer = EditorState::new();
        assert!(empty_buffer.is_cursor_at_last_line());
    }

    #[test]
    fn test_is_cursor_at_end() {
        let mut buffer = EditorState::new();
        buffer.push("Line 1");
        buffer.push("Line 2");
        buffer.set_cursor_position(1, 5);
        assert!(buffer.is_cursor_at_end());

        buffer.set_cursor_position(0, 5);
        assert!(!buffer.is_cursor_at_end());

        buffer.set_cursor_position(1, 2);
        assert!(!buffer.is_cursor_at_end());

        let empty_buffer = EditorState::new();
        assert!(empty_buffer.is_cursor_at_end());
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = EditorState::new();
        buffer.push("Line 1");

        buffer.set_cursor_position(0, 1);
        buffer.insert_char('a');

        let exp: Vec<Line> = vec![Line::from("Laine 1")];
        assert_eq!(buffer.lines().to_vec(), exp);
        assert_eq!(buffer.cursor, Position::new(0, 2));
    }

    #[test]
    fn test_insert_newline() {
        let mut buffer = EditorState::new();
        buffer.set_cursor_position(0, 1);
        buffer.push("Line 1");

        buffer.insert_newline();

        let exp: Vec<Line> = vec![Line::from("L"), Line::from("ine 1")];
        assert_eq!(buffer.lines().to_vec(), exp);

        assert_eq!(buffer.cursor, Position::new(1, 0));
    }

    #[test]
    fn test_select_between() {
        let mut buffer = EditorState::new();
        buffer.set_cursor_position(0, 4);
        buffer.push("L \"ine\" 1");
        buffer.select_between_delimiters(&vec!['"']);

        buffer.insert_newline();

        assert_eq!(
            buffer.selection,
            Some(Selection {
                start: Position::new(0, 3),
                end: Position::new(0, 5),
            })
        );

        assert_eq!(buffer.cursor, Position::new(1, 0));
    }

    #[test]
    fn test_text_between_selection() {
        let mut buffer = EditorState::new();
        buffer.push("Line 1");
        buffer.push("Line 2");
        buffer.selection = Some(Selection {
            start: Position { line: 0, column: 2 },
            end: Position { line: 1, column: 3 },
        });
        let mut buffer_exp = EditorState::new();
        buffer_exp.push("e 2");

        let text = buffer.text_between_selection();
        assert_eq!(text, "ne 1\nLine");
    }
}

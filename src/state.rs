pub mod mode;
pub mod position;
pub mod selection;
mod undo;
mod view;

use std::char;

use self::view::ViewState;
use self::{mode::EditorMode, position::Position, selection::Selection, undo::Stack};
use crate::Lines;
use jagged::index::RowIndex;
use jagged::{Index2, Jagged};

#[derive(Clone)]
pub struct EditorState {
    pub lines: Lines,
    pub cursor: Position,
    pub mode: EditorMode,
    pub selection: Option<Selection>,
    pub(crate) view: ViewState,
    pub(crate) undo: Stack,
    pub(crate) redo: Stack,
}

impl Default for EditorState {
    fn default() -> Self {
        EditorState::new()
    }
}

impl EditorState {
    /// Create a new editor state.
    #[must_use]
    pub fn new() -> EditorState {
        EditorState {
            lines: Jagged::default(),
            cursor: Position::new(0, 0),
            mode: EditorMode::Normal,
            selection: None,
            view: ViewState::default(),
            undo: Stack::new(),
            redo: Stack::new(),
        }
    }

    /// Switch to normal mode
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
        self.set_selection(self.cursor, self.cursor);
    }

    /// Switch to insert mode and move one character forward
    pub fn append_mode(&mut self) {
        self.mode = EditorMode::Insert;
        if self.cursor.column < self.len_col() {
            self.cursor.column += 1;
        }
    }

    /// Clears the buffer
    pub fn clear(&mut self) {
        self.lines.clear();
        self.cursor = Position::new(0, 0);
        self.selection = None;
    }

    /// Appends a line to the back of the buffer.
    pub fn push(&mut self, line: &str) {
        let chars: Vec<char> = line.chars().collect();
        self.lines.push(chars);
    }

    /// Inserts a line at a given index to the back of the buffer.
    pub fn insert(&mut self, index: usize, line: &str) {
        let chars: Vec<char> = line.chars().collect();
        self.lines.insert(RowIndex::new(index), chars);
    }

    /// Inserts a single character at the current cursor position
    pub fn insert_char(&mut self, ch: char) {
        if self.lines.is_empty() {
            self.lines.push(Vec::new());
        }
        if ch == '\n' {
            self.insert_newline();
        } else {
            self.lines.insert(self.cursor.as_index(), ch);
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
        let at = Index2::new(self.cursor.line, self.cursor.column);
        let mut rest = self.lines.split_off(at);
        self.lines.append(&mut rest);
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

    /// Set the selection start positions
    pub fn set_selection_start(&mut self, start: Position) {
        if let Some(end) = self.selection.as_ref().map(|x| x.end) {
            self.selection = Some(Selection { start, end });
        }
    }

    /// Set the selection end positions
    pub fn set_selection_end(&mut self, end: Position) {
        if let Some(start) = self.selection.as_ref().map(|x| x.start) {
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
            self.cursor.column = self.len_col();
        }

        if self.mode == EditorMode::Visual {
            let cursor = self.cursor;
            self.set_selection_end(cursor);
        }
    }

    /// Move the cursor right by one column, wrapping to the next line if needed
    pub fn move_cursor_right(&mut self) {
        if self.cursor.column < self.len_col() {
            self.cursor.column += 1;
        } else if self.cursor.line + 1 < self.len() {
            self.cursor.line += 1;
            self.cursor.column = 0;
        }

        if self.mode == EditorMode::Visual {
            let cursor = self.cursor;
            self.set_selection_end(cursor);
        }
    }

    /// Move the cursor up by one line
    pub fn move_cursor_up(&mut self) {
        if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.column = self.cursor.column.min(self.len_col());
        }

        if self.mode == EditorMode::Visual {
            let cursor = self.cursor;
            self.set_selection_end(cursor);
        }
    }

    /// Move the cursor down by one line
    pub fn move_cursor_down(&mut self) {
        if !self.lines.is_empty() && self.cursor.line < self.len() - 1 {
            self.cursor.line += 1;
            self.cursor.column = self.cursor.column.min(self.len_col());
        }

        if self.mode == EditorMode::Visual {
            let cursor = self.cursor;
            self.set_selection_end(cursor);
        }
    }

    /// Move the cursor to the beginning of the line
    pub fn move_cursor_to_beginning_of_line(&mut self) {
        self.cursor.column = 0;

        if self.mode == EditorMode::Visual {
            let cursor = self.cursor;
            self.set_selection_end(cursor);
        }
    }

    /// Move the cursor to the end of the line
    pub fn move_cursor_to_end_of_line(&mut self) {
        self.cursor.column = self.len_col().saturating_sub(1);

        if self.mode == EditorMode::Visual {
            let cursor = self.cursor;
            self.set_selection_end(cursor);
        }
    }

    /// Move the cursor right by one word, wrapping to the next line if needed
    pub fn move_cursor_one_word_right(&mut self) {
        let current_index = self.cursor.as_index();
        if let Some(current_char) = self.get(current_index) {
            // Find the index of the next non-whitespace character
            if let Some((_, next_index)) = self.lines.next_predicate(current_index, |ch| {
                !Self::is_same_word_class(*current_char, *ch) && !ch.is_whitespace()
            }) {
                self.cursor = next_index.into();
            } else {
                // If no word boundary is found, move to the end
                self.cursor = self.last_pos();
            }
        }
    }

    ///  Move the cursor left by one word, wrapping to the next line if ,needed
    pub fn move_cursor_one_word_left(&mut self) {
        // Always move one character to the left
        self.cursor.column = self.cursor.column.saturating_sub(1);

        let current_index = self.cursor.as_index();
        if let Some(current_char) = self.get(current_index) {
            if let Some((_, index)) = self.lines.prev_predicate(current_index, |ch| {
                !Self::is_same_word_class(*current_char, *ch)
            }) {
                if index.col != 0 && !current_char.is_whitespace() {
                    self.cursor = Position::new(index.row, index.col + 1);
                } else {
                    self.cursor = index.into();
                }
            } else {
                self.cursor = Position::default();
            }
        }
    }

    /// Deletes a single character at the current cursor position. Does
    /// not move the cursor position unless it is at the end of the line.
    pub fn remove_char(&mut self) {
        if self.len_col() == 0 {
            return;
        }
        let _ = self.lines.remove(self.cursor.as_index());
        self.cursor.column = self.cursor.column.min(self.len_col().saturating_sub(1));
    }

    /// Deletes a single character at the current cursor position. Moves the cursor
    /// position by one and jumps to the previous line if necessary.
    pub fn delete_char(&mut self) {
        if self.cursor.column == 0 && self.cursor.line == 0 {
            return;
        }

        if self.cursor.column == 0 {
            let mut rest = self.lines.split_off(self.cursor.as_index());
            self.move_cursor_left();
            self.lines.merge(&mut rest);
        } else {
            self.move_cursor_left();
            let _ = self.lines.remove(self.cursor.as_index());
        }
    }

    /// Delete the current line.
    pub fn delete_line(&mut self) {
        if self.cursor.line < self.lines.len() {
            self.lines.remove(RowIndex::new(self.cursor.line));
            self.cursor.column = 0;
            self.cursor.line = self.cursor.line.min(self.lines.len().saturating_sub(1));
        }
    }

    /// Delete between a start and end position.
    fn delete_between(&mut self, start: &Position, end: &Position) {
        self.cursor = *end;
        // TODO: Implement a better way to clear selection
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
            // TODO: Implement a better way to clear selection
            self.move_cursor_right();
            self.delete_char();
            while self.cursor != selection.start() {
                self.delete_char();
            }
        }
    }

    /// Selects text between specified delimiter characters.
    ///
    /// This function takes an input text and a list of delimiter characters as input.
    /// It searches for the first occurrence of a delimiter character in the text to
    /// define the start of the selection, and the next occurrence of any of the delimiter
    /// characters to define the end of the selection.
    pub fn select_between_delimiters(&mut self, delimiters: &[char]) {
        let cursor = self.cursor;
        let mut start: Option<Index2> = None;
        let mut end: Option<Index2> = None;
        let mut prev = cursor.as_index();
        for (value, index) in self.lines.iter().from(cursor.as_index()) {
            if let Some(c) = value {
                if delimiters.contains(c) {
                    end = Some(prev);
                    break;
                }
            }
            prev = index;
        }
        prev = cursor.as_index();
        for (value, index) in self.lines.iter().from(cursor.as_index()).rev() {
            if let Some(c) = value {
                if delimiters.contains(c) {
                    start = Some(prev);
                    break;
                }
            }
            prev = index;
        }
        if let (Some(start), Some(end)) = (start, end) {
            self.selection = Some(Selection {
                start: start.into(),
                end: end.into(),
            });
        }
    }

    /// Get the number of rows.
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.lines.len()
    }

    /// Get the number of columns in the current line.
    #[must_use]
    pub(crate) fn len_col(&self) -> usize {
        self.len_col_at(self.cursor.line)
    }

    /// Get the number of columns of a line from the line index.
    #[must_use]
    pub fn len_col_at(&self, index: usize) -> usize {
        match self.lines.get(RowIndex::new(index)) {
            Some(line) => line.len(),
            None => 0,
        }
    }

    /// Get the curently selected character.
    #[must_use]
    pub fn char(&self) -> Option<&char> {
        self.lines.get(self.cursor.as_index())
    }

    /// Wwhether the cursor is at the of the current line.
    #[must_use]
    pub fn is_cursor_at_end_of_line(&self) -> bool {
        self.len_col() <= self.cursor.column + 1
    }

    /// Wwhether the cursor is at the of the current line.
    #[must_use]
    pub fn is_cursor_at_start_of_line(&self) -> bool {
        self.cursor.column == 0
    }

    /// Wwhether the cursor is at the last line.
    #[must_use]
    pub fn is_cursor_at_last_line(&self) -> bool {
        self.len() <= self.cursor.line + 1
    }

    /// Whether the cursor is at the end of the buffer.
    #[must_use]
    pub fn is_cursor_at_end(&self) -> bool {
        self.is_cursor_at_end_of_line() && self.is_cursor_at_last_line()
    }

    /// Whether the cursor is at the beginning of the buffer.
    #[must_use]
    pub fn is_cursor_at_start(&self) -> bool {
        self.cursor.line == 0 && self.cursor.column == 0
    }

    /// Whether two characters are considered of the same class.
    #[must_use]
    fn is_same_word_class(a: char, b: char) -> bool {
        a.is_ascii_alphanumeric() && b.is_ascii_alphanumeric()
            || (a.is_ascii_punctuation() && b.is_ascii_punctuation())
            || (a.is_ascii_whitespace() && b.is_ascii_whitespace())
    }

    fn last_pos(&self) -> Position {
        let row = self.len().saturating_sub(1);
        Position::new(row, self.len_col_at(row).saturating_sub(1))
    }

    fn get(&self, index: Index2) -> Option<&char> {
        self.lines.get(index)
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
        buffer.push(" Boom...");

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
        assert_eq!(buffer.cursor.column, 7);

        buffer.move_cursor_one_word_right();
        assert_eq!(buffer.cursor.line, 1);
        assert_eq!(buffer.cursor.column, 7);
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
        expect = Jagged::from("Line1");
        assert_eq!(buffer.lines, expect);
        assert_eq!(buffer.cursor.column, 4);

        buffer.remove_char();
        expect = Jagged::from("Line");
        assert_eq!(buffer.lines, expect);
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

        expect = Jagged::from(
            "Lne 1\n\
             \n\
             Line 3",
        );
        // vec![vec!["Lne 1"], vec![""], vec!["Line 3"]]);
        assert_eq!(buffer.lines, expect);
        assert_eq!(buffer.cursor, Position::new(0, 1));

        buffer.set_cursor_position(2, 2);
        for _ in 0..4 {
            buffer.delete_char();
        }

        expect = Jagged::from("Lne 1ne 3");
        assert_eq!(buffer.lines, expect);
        assert_eq!(buffer.cursor, Position::new(0, 5));

        // buffer.set_cursor_position(0, 9);
        // buffer.delete_char();
        //
        // expect = Jagged::from("Lne 1ne ");
        // assert_eq!(buffer.lines2, expect);
        // assert_eq!(buffer.cursor, Position::new(0, 8));
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

        // let exp: Vec<Line> = vec![Line::from("Laine 1")];
        // assert_eq!(buffer.lines().to_vec(), exp);

        let exp: Jagged<char> = Jagged::from("Laine 1");
        assert_eq!(buffer.lines, exp);

        assert_eq!(buffer.cursor, Position::new(0, 2));
    }

    #[test]
    fn test_insert_newline() {
        // given
        let mut buffer = EditorState::new();
        buffer.set_cursor_position(0, 1);
        buffer.push("Line 1");

        // when
        buffer.insert_newline();

        // then
        let exp: Jagged<char> = Jagged::from("L\nine 1");
        assert_eq!(buffer.lines, exp);
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

    // #[test]
    // fn test_text_between_selection() {
    //     let mut buffer = EditorState::new();
    //     buffer.push("Line 1");
    //     buffer.push("Line 2");
    //     buffer.selection = Some(Selection {
    //         start: Position { line: 0, column: 2 },
    //         end: Position { line: 1, column: 3 },
    //     });
    //     let mut buffer_exp = EditorState::new();
    //     buffer_exp.push("e 2");
    //
    //     let text = buffer.text_between_selection();
    //     assert_eq!(text, "ne 1\nLine");
    // }
}

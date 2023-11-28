#![allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub mod iterator;
use std::{
    borrow::Cow,
    char,
    ops::{Deref, DerefMut},
};

pub use self::iterator::LinesIterator;
use crate::state::position::Position;

/// We use a vector of characters instead of a string. The problem with
/// strings is that they cannot be unambigously indexed, which is
/// necessary in some cases.
#[derive(Debug, Clone, PartialEq)]
pub struct Line(Vec<char>);

impl Deref for Line {
    type Target = Vec<char>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Line {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<&[char]> for Line {
    fn from(ch: &[char]) -> Self {
        Line(ch.iter().map(Clone::clone).collect())
    }
}
impl From<Vec<char>> for Line {
    fn from(ch: Vec<char>) -> Self {
        Line(ch.into_iter().collect())
    }
}

impl From<String> for Line {
    fn from(s: String) -> Self {
        Line(s.chars().collect())
    }
}

impl<'a> From<&'a Line> for String {
    fn from(l: &'a Line) -> Self {
        l.iter().collect()
    }
}

impl From<&str> for Line {
    fn from(s: &str) -> Self {
        Line(s.chars().collect())
    }
}

impl<'a> From<&'a Line> for Cow<'a, str> {
    fn from(l: &'a Line) -> Self {
        Cow::Owned(l.into())
    }
}

impl Line {
    pub fn push_chars(&mut self, chars: &Line) {
        self.0.extend_from_slice(chars);
    }
    pub fn to_string(&self) -> String {
        self.into()
    }
}

impl Line {
    /// Returns the column of the next word.
    #[must_use]
    pub fn next_word(&self, pos: usize) -> usize {
        let mut pos = pos;
        let Some(first_char) = self.char_at(pos) else {
            return pos;
        };

        // Always move one character at least
        pos += 1;

        // Move to the next character of a different class then the first.
        while let Some(c) = self.char_at(pos) {
            if !Self::is_same_word_class(*first_char, *c) {
                break;
            }
            pos += 1;
        }

        // Move to the next non-blank character.
        pos = self.skip_whitespace(pos);

        // Set back on if out of bounds
        self.len().saturating_sub(1).min(pos)
    }

    /// Returns the column of the previous word.
    #[must_use]
    pub fn prev_word(&self, pos: usize) -> usize {
        let mut pos = pos.saturating_sub(1);
        let Some(first_char) = self.char_at(pos) else {
            return pos;
        };

        // Move to the next character of a different class then the first.
        while let Some(c) = self.char_at(pos) {
            if !Self::is_same_word_class(*first_char, *c) || pos == 0 {
                break;
            }
            pos -= 1;
        }

        // Move cursor one character forward if necessary.
        if pos != 0 && !first_char.is_ascii_whitespace() {
            pos += 1;
        }

        pos
    }

    /// Get the curently selected character.
    #[must_use]
    pub fn char_at(&self, pos: usize) -> Option<&char> {
        self.iter().nth(pos)
    }

    /// Returns the column of the next character that is not a whitespace.
    #[must_use]
    pub fn skip_whitespace(&self, pos: usize) -> usize {
        let mut pos = pos;
        while let Some(c) = self.char_at(pos) {
            if !c.is_ascii_whitespace() {
                break;
            }
            pos += 1;
        }
        pos
    }

    /// Whether two characters are considered of the same class.
    fn is_same_word_class(a: char, b: char) -> bool {
        a.is_ascii_alphanumeric() && b.is_ascii_alphanumeric()
            || (a.is_ascii_punctuation() && b.is_ascii_punctuation())
            || (a.is_ascii_whitespace() && b.is_ascii_whitespace())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Lines(Vec<Line>);

impl Deref for Lines {
    type Target = Vec<Line>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Lines {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<String> for Lines {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl From<&str> for Lines {
    fn from(s: &str) -> Self {
        Self(
            s.lines()
                .map(|line| Line::from(line))
                .collect::<Vec<Line>>(),
        )
    }
}

impl From<Vec<Line>> for Lines {
    fn from(lines: Vec<Line>) -> Self {
        Self(lines)
    }
}

impl Lines {
    /// Constructs an empty vector of lines
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }
    /// Returns [`Lines`] as a string object
    #[must_use]
    pub fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("\n")
        // self.0.join("\n")
    }

    /// Appends a line to the back of the buffer.
    pub fn push<T>(&mut self, line: T)
    where
        T: Into<Line>,
    {
        self.0.push(line.into());
    }

    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    pub fn append(&mut self, other: &mut Self) {
        if other.0.is_empty() {
            return;
        }
        let last_line = self.len().saturating_sub(1);
        self.0[last_line].append(&mut other.remove(0));
        self.0.append(&mut other.0);
    }

    /// Splits a [`Lines`] into two at a the given [`Position`].
    ///
    /// Returns a newly allocated vector containing the elements in the range
    /// `[at, len)`. After the call, the original vector will be left containing
    /// the elements `[0, at)` with its previous capacity unchanged.
    pub fn split_off(&mut self, at: &Position) -> Self {
        if at.column == 0 {
            return self.0.split_off(at.line).into();
        } else {
            let a = self.0.remove(at.line);
            let (left, right) = a.split_at(at.column);
            let mut last = self.0.split_off(at.line);
            self.0.push(Line::from(left));
            last.insert(0, Line::from(right));
            return last.into();
        }
    }

    /// Return the cursor position of the next word. Stops at the end
    /// of the line.
    #[must_use]
    pub fn next_word(&self, pos: &Position) -> Position {
        let new_pos = self[pos.line].next_word(pos.column);
        Position::new(pos.line, new_pos)
    }

    /// Return the cursor position of the previous word. Stops at the beginning
    /// of the line.
    #[must_use]
    pub fn prev_word(&self, pos: &Position) -> Position {
        let new_pos = self[pos.line].prev_word(pos.column);
        Position::new(pos.line, new_pos)
    }

    /// Returns a cursor that skips all whitespace characters moving forward.
    #[must_use]
    pub fn skip_whitespace(&self, pos: &Position) -> Position {
        let new_pos = self[pos.line].skip_whitespace(pos.column);
        Position::new(pos.line, new_pos)
    }

    /// Splits a line into two at a given cursor position. Has the same
    /// effect as inserting a newline.
    pub fn insert_newline(&mut self, cursor: &Position) {
        // Use split_off to split the line at cursor.column
        let b = self[cursor.line].split_off(cursor.column);
        // Insert the new line after cursor.line
        self.insert(cursor.line + 1, Line(b));
        // Truncate the original line at cursor.column
        self[cursor.line].truncate(cursor.column);
    }

    /// Removes the text between two positions. Returns the deleted
    /// object as a new [`Lines`].
    pub fn drain(&mut self, start: &Position, end: &Position) -> Self {
        // If start and end are both at the beginning of the line, we
        // can simply drain the llines.
        if start.column == 0 && end.column == 0 {
            let drained = self.0.drain(start.line..end.line);
            return Lines::from(drained.collect::<Vec<Line>>());
        }
        // Oterwise we need to split out the text between and start and
        // merge the lines afterwards.
        let mut b = self.split_off(start);
        let mut c = b.split_off(end);
        self.append(&mut c);
        return b;
    }

    /// If the cursor is at the end of the line, this method wraps it to the next
    /// line and skips non blank characters if necessary.
    #[must_use]
    pub fn wrap_forward(&self, pos: &Position) -> Position {
        if !self.is_end_of_line(pos) || self.is_last_line(pos) {
            return pos.clone();
        }
        self.skip_whitespace(&Position::new(pos.line + 1, 0))
    }

    /// Get the curently selected character.
    #[must_use]
    pub fn char_at(&self, pos: &Position) -> Option<&char> {
        let line = &self[pos.line];
        line.iter().nth(pos.column)
    }

    // Whether the cursor is at the start of the line
    #[must_use]
    fn is_start_of_line(&self, pos: &Position) -> bool {
        pos.column == 0
    }

    // Whether the cursor is at the end of the line
    #[must_use]
    fn is_end_of_line(&self, pos: &Position) -> bool {
        self.column_len_at(pos.line) <= pos.column + 1
    }

    // Whether the cursor is at the last line
    #[must_use]
    fn is_last_line(&self, pos: &Position) -> bool {
        self.len() <= pos.line + 1
    }

    /// Get the length of a line at a given index. Returns 0 if the
    /// index is out of bounds.
    #[must_use]
    pub fn column_len_at(&self, index: usize) -> usize {
        match self.get(index) {
            Some(line) => line.len(),
            None => 0,
        }
    }
    /// Returns an iterator that yields [`Position`] objects. The iterator
    /// starts from a given position.
    #[must_use]
    pub fn pos_iter(&self) -> LinesIterator {
        LinesIterator {
            lines: self,
            line: 0,
            column: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::position::Position;

    use super::*;

    #[test]
    fn test_move_one_word_right() {
        let mut lines = Lines::new();
        lines.push("Hello, world ");
        lines.push("Boom.");

        let pos = lines.next_word(&Position::new(0, 0));
        assert_eq!(pos, Position::new(0, 5));

        let pos = lines.next_word(&Position::new(0, 5));
        assert_eq!(pos, Position::new(0, 7));

        let pos = lines.next_word(&Position::new(0, 7));
        assert_eq!(pos, Position::new(0, 12));

        let pos = lines.next_word(&Position::new(0, 12));
        assert_eq!(pos, Position::new(0, 12));
    }

    #[test]
    fn test_move_one_word_left() {
        let mut lines = Lines::new();
        lines.push("Hello, world! Boom.");

        let pos = lines.prev_word(&Position::new(0, 18));
        assert_eq!(pos, Position::new(0, 14));

        let pos = lines.prev_word(&Position::new(0, 14));
        assert_eq!(pos, Position::new(0, 12));

        let pos = lines.prev_word(&Position::new(0, 12));
        assert_eq!(pos, Position::new(0, 7));

        let pos = lines.prev_word(&Position::new(0, 7));
        assert_eq!(pos, Position::new(0, 5));

        let pos = lines.prev_word(&Position::new(0, 5));
        assert_eq!(pos, Position::new(0, 0));

        let pos = lines.prev_word(&Position::new(0, 0));
        assert_eq!(pos, Position::new(0, 0));
    }

    #[test]
    fn test_wrap_forward() {
        let mut lines = Lines::new();
        lines.push("Hello, world!");
        lines.push(" Boom.");

        let pos = lines.wrap_forward(&Position::new(0, 4));
        assert_eq!(pos, Position::new(0, 4));

        let pos = lines.wrap_forward(&Position::new(0, 12));
        assert_eq!(pos, Position::new(1, 1));
    }

    #[test]
    fn test_append() {
        let mut lines1 = Lines::new();
        lines1.push("Hello,");
        let mut lines2 = Lines::new();
        lines2.push(" world");
        lines2.push("Boom.");

        lines1.append(&mut lines2);
        assert_eq!(lines1, Lines::from("Hello, world\nBoom."));
    }

    #[test]
    fn test_split_off() {
        let mut lines = Lines::from("Hello, world\nBoom.");

        let split = lines.split_off(&Position::new(0, 5));
        assert_eq!(lines, Lines::from("Hello"));
        assert_eq!(split, Lines::from(", world\nBoom."));

        let mut lines = Lines::from("Hello, world\nBoom.");
        let split = lines.split_off(&Position::new(1, 0));
        assert_eq!(lines, Lines::from("Hello, world"));
        assert_eq!(split, Lines::from("Boom."));
    }

    #[test]
    fn test_drain() {
        let mut lines = Lines::from("Hello, world\nBoom.");
        let drained = lines.drain(&Position::new(0, 6), &Position::new(1, 1));
        assert_eq!(lines, Lines::from("Hello,oom."));
        assert_eq!(drained, Lines::from(" world\nB"));

        let mut lines = Lines::from("Hello, world\nBoom\nBoom.");
        let drained = lines.drain(&Position::new(1, 0), &Position::new(2, 0));
        assert_eq!(lines, Lines::from("Hello, world\nBoom."));
        assert_eq!(drained, Lines::from("Boom"));
    }
}

use std::iter::FusedIterator;

use crate::{state::position::Position, Lines};

/// An iterator for [`Lines`] that yields [`Position`] objects.
#[derive(Clone, Debug)]
pub struct LinesIterator<'a> {
    pub(super) lines: &'a Lines,
    pub(super) line: isize,
    pub(super) column: usize,
}

impl<'a> LinesIterator<'a> {
    /// A [`LinesIterator`] that starts from a given position.
    #[must_use]
    pub fn start(self, pos: &Position) -> Self {
        Self {
            lines: self.lines,
            line: pos.line as isize,
            column: pos.column,
        }
    }

    /// A [`LinesTakeUntil`] iterator that iterates until a given
    /// precondition is met.
    pub fn take_until<P>(self, predicate: P) -> LinesTakeUntilIterator<'a, P>
    where
        P: FnMut(&<Self as Iterator>::Item) -> bool,
    {
        LinesTakeUntilIterator {
            iter: self,
            predicate,
            flag: false,
        }
    }
}

impl<'a> Iterator for LinesIterator<'a> {
    type Item = Position;

    fn next(&mut self) -> Option<Position> {
        if self.line as usize >= self.lines.len() {
            return None; // Reached the end of lines, stop iterating
        }

        let current = Position::new(self.line as usize, self.column);
        let lines = &self.lines;
        if self.column < lines.column_len_at(self.line as usize).saturating_sub(1) {
            // Move to the next column within the current line
            self.column += 1;
        } else {
            // If it's the last column, move to the next line
            self.line += 1;
            // Skip empty lines
            while !lines.is_last_line(&Position::new(self.line as usize, self.column))
                && self.lines.column_len_at(self.line as usize) == 0
            {
                self.line += 1;
            }
            self.column = 0;
        }

        Some(current)
    }
}

impl<'a> DoubleEndedIterator for LinesIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.line < 0 {
            return None; // Reached the beginning of the buffer, stop iterating
        }
        let current = Position::new(self.line as usize, self.column);

        if self.column > 0 {
            // Move to the previous column within the current line
            self.column = self.column.saturating_sub(1);
        } else {
            // If it's the first column, move to the next line
            self.line -= 1;
            // Skip empty lines
            while self.line >= 0 && self.lines.column_len_at(self.line as usize) == 0 {
                self.line -= 1;
            }
            self.column = self
                .lines
                .column_len_at(self.line as usize)
                .saturating_sub(1);
        }

        Some(current)
    }
}
impl<'a> FusedIterator for LinesIterator<'a> {}

/// An [`Iterator`] that iterates [`LinesIterator`] until a precondition is met.
/// [`LinesTakeUntilIterator`] is similar to take while Iterator but takes
/// elements including the last element where precondition is true.
///
/// [`LinesTakeUntilIterator`] is created by the [`take_until`] method on [`LinesIterator`].
#[derive(Clone, Debug)]
pub struct LinesTakeUntilIterator<'a, P> {
    iter: LinesIterator<'a>,
    flag: bool,
    predicate: P,
}

impl<'a, P> Iterator for LinesTakeUntilIterator<'a, P>
where
    P: FnMut(&<LinesIterator as Iterator>::Item) -> bool,
{
    type Item = <LinesIterator<'a> as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.flag {
            None
        } else {
            self.iter.next().map(|x| {
                if (self.predicate)(&x) {
                    self.flag = true;
                }
                x
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.flag {
            (0, Some(0))
        } else {
            (0, self.iter.size_hint().1)
        }
    }
}

impl<'a, P> FusedIterator for LinesTakeUntilIterator<'a, P> where
    P: FnMut(&<LinesIterator as Iterator>::Item) -> bool
{
}

#[cfg(test)]
mod tests {
    use crate::state::position::Position;

    use super::*;

    #[test]
    fn test_iter() {
        let mut lines = Lines::new();
        lines.push("He");
        lines.push("");
        lines.push("B");
        let cursors: Vec<Position> = lines.pos_iter().collect();

        assert_eq!(
            cursors,
            vec![
                Position::new(0, 0),
                Position::new(0, 1),
                Position::new(2, 0),
            ]
        );

        let mut lines = Lines::new();
        lines.push("H");
        lines.push("");
        lines.push("");
        let cursors: Vec<Position> = lines.pos_iter().collect();

        assert_eq!(cursors, vec![Position::new(0, 0), Position::new(2, 0),]);
    }

    #[test]
    fn test_iter_rev() {
        let start = Position::new(2, 0);
        let mut lines = Lines::new();
        lines.push("He");
        lines.push("");
        lines.push("B");
        let cursors: Vec<Position> = lines.pos_iter().start(&start).rev().collect();

        assert_eq!(
            cursors,
            vec![
                Position::new(2, 0),
                Position::new(0, 1),
                Position::new(0, 0),
            ]
        );

        let mut lines = Lines::new();
        lines.push("H");
        lines.push("");
        lines.push("");
        let cursors: Vec<Position> = lines.pos_iter().start(&start).rev().collect();

        assert_eq!(cursors, vec![Position::new(2, 0), Position::new(0, 0),]);
    }

    #[test]
    fn test_iter_take_until() {
        let mut lines = Lines::new();
        lines.push("He");
        lines.push("");
        lines.push("Wo");
        let cursors: Vec<Position> = lines
            .pos_iter()
            .start(&Position::new(0, 1))
            .take_until(|pos| *pos == Position::new(2, 0))
            .collect();

        assert_eq!(cursors, vec![Position::new(0, 1), Position::new(2, 0),]);
    }
}

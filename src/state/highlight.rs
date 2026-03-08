//! Custom highlight ranges for the editor.

use crate::Index2;
use ratatui_core::style::Style;

/// A highlighted range in the editor with a custom style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Highlight {
    /// Start position (row, column).
    pub start: Index2,
    /// End position (row, column), inclusive.
    pub end: Index2,
    /// Style to apply to the highlighted range.
    pub style: Style,
}

impl Highlight {
    #[must_use]
    pub fn new(start: Index2, end: Index2, style: Style) -> Self {
        Self { start, end, style }
    }

    #[must_use]
    pub fn contains(&self, pos: &Index2) -> bool {
        let (start, end) = if self.start < self.end {
            (&self.start, &self.end)
        } else {
            (&self.end, &self.start)
        };

        match (pos.row, pos.col) {
            (row, _) if row > start.row && row < end.row => true,
            (row, col) if row > start.row && row == end.row => col <= end.col,
            (row, col) if row == start.row && row < end.row => col >= start.col,
            (row, col) if row == start.row && row == end.row => col >= start.col && col <= end.col,
            _ => false,
        }
    }

    #[must_use]
    pub fn start(&self) -> Index2 {
        if self.is_reversed() {
            self.end
        } else {
            self.start
        }
    }

    #[must_use]
    pub fn end(&self) -> Index2 {
        if self.is_reversed() {
            self.start
        } else {
            self.end
        }
    }

    #[must_use]
    fn is_reversed(&self) -> bool {
        self.start.row > self.end.row
            || (self.start.row == self.end.row && self.start.col > self.end.col)
    }

    #[must_use]
    pub fn contains_row(&self, row: usize) -> bool {
        let (start_row, end_row) = if self.start.row < self.end.row {
            (self.start.row, self.end.row)
        } else {
            (self.end.row, self.start.row)
        };
        row >= start_row && row <= end_row
    }

    #[must_use]
    pub fn get_columns_in_row(&self, row: usize, row_len: usize) -> Option<(usize, usize)> {
        let (start, end) = (self.start(), self.end());

        if row < start.row || row > end.row {
            return None;
        }

        let start_col = if row == start.row {
            start.col.min(row_len)
        } else {
            0
        };

        let end_col = if row == end.row {
            end.col.min(row_len)
        } else {
            row_len
        };

        Some((start_col, end_col))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_core::style::Color;

    #[test]
    fn test_highlight_contains() {
        let highlight = Highlight::new(Index2::new(0, 2), Index2::new(0, 5), Style::default());

        assert!(!highlight.contains(&Index2::new(0, 1)));
        assert!(highlight.contains(&Index2::new(0, 2)));
        assert!(highlight.contains(&Index2::new(0, 3)));
        assert!(highlight.contains(&Index2::new(0, 5)));
        assert!(!highlight.contains(&Index2::new(0, 6)));
    }

    #[test]
    fn test_highlight_multiline() {
        let highlight = Highlight::new(Index2::new(0, 5), Index2::new(2, 3), Style::default());

        assert!(!highlight.contains(&Index2::new(0, 4)));
        assert!(highlight.contains(&Index2::new(0, 5)));
        assert!(highlight.contains(&Index2::new(1, 0)));
        assert!(highlight.contains(&Index2::new(1, 100)));
        assert!(highlight.contains(&Index2::new(2, 0)));
        assert!(highlight.contains(&Index2::new(2, 3)));
        assert!(!highlight.contains(&Index2::new(2, 4)));
    }

    #[test]
    fn test_highlight_columns_in_row() {
        let highlight = Highlight::new(
            Index2::new(1, 3),
            Index2::new(3, 7),
            Style::default().bg(Color::Yellow),
        );

        assert_eq!(highlight.get_columns_in_row(0, 10), None);
        assert_eq!(highlight.get_columns_in_row(1, 10), Some((3, 10)));
        assert_eq!(highlight.get_columns_in_row(2, 10), Some((0, 10)));
        assert_eq!(highlight.get_columns_in_row(3, 10), Some((0, 7)));
        assert_eq!(highlight.get_columns_in_row(4, 10), None);
    }
}

use std::cmp::Ordering;

use crate::{Index2, Lines};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub start: Index2,
    pub end: Index2,
}

impl Selection {
    #[must_use]
    pub fn new(start: Index2, end: Index2) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub fn contains(&self, pos: &Index2) -> bool {
        let (start, end) = if self.start < self.end {
            (&self.start, &self.end)
        } else {
            (&self.end, &self.start)
        };
        let (st_row, st_col) = (start.row, start.col);
        let (en_row, en_col) = (end.row, end.col);

        match (pos.row, pos.col) {
            (line, _) if line > st_row && line < en_row => true,
            (line, column) if line > st_row && line == en_row => column <= en_col,
            (line, column) if line == st_row && line < en_row => column >= st_col,
            (line, column) if line == st_row && line == en_row => {
                column <= en_col && column >= st_col
            }
            _ => false,
        }
    }

    #[must_use]
    pub fn contains_row(&self, row_index: usize) -> bool {
        let (start_row, end_row) = if self.start.row < self.end.row {
            (self.start.row, self.end.row)
        } else {
            (self.end.row, self.start.row)
        };
        if row_index >= start_row && row_index <= end_row {
            return true;
        }
        false
    }

    #[must_use]
    pub fn start(&self) -> Index2 {
        if self.is_reversed() {
            return self.end;
        }
        self.start
    }

    #[must_use]
    pub fn end(&self) -> Index2 {
        if self.is_reversed() {
            return self.start;
        }
        self.end
    }

    #[must_use]
    fn is_reversed(&self) -> bool {
        self.start.row > self.end.row
            || self.start.row == self.end.row && self.start.col > self.end.col
    }

    fn reverse(&mut self) {
        (self.start, self.end) = (self.end, self.start);
    }

    /// Extracts a selection from `Lines`.
    #[must_use]
    pub fn extract(&self, lines: &Lines) -> Lines {
        lines.iter().from(self.start()).to(self.end()).collect()
    }

    /// Returns the start and end column of the selection in the given row.
    /// If the selection does not intersect with the row, the function returns None.
    #[must_use]
    pub fn selected_columns_in_row(
        &self,
        row_len: usize,
        row_index: usize,
    ) -> Option<(usize, usize)> {
        let (start, end) = (self.start(), self.end());

        let start_col = match start.row.cmp(&row_index) {
            Ordering::Less => 0,
            Ordering::Greater => {
                return None;
            }
            Ordering::Equal => start.col.min(row_len),
        };

        let end_col = match end.row.cmp(&row_index) {
            Ordering::Less => {
                return None;
            }
            Ordering::Greater => row_len,
            Ordering::Equal => end.col.min(row_len),
        };

        Some((start_col, end_col))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_data() -> Lines {
        Lines::from(
            "Hello\n\
            World",
        )
    }

    #[test]
    fn test_extract() {
        let data = test_data();
        let selection = Selection::new(Index2::new(0, 3), Index2::new(1, 1));

        assert_eq!(selection.extract(&data), Lines::from("lo\nWo"));
    }

    #[test]
    fn test_selection_columns_in_row() {
        // given
        let selection = Selection::new(Index2::new(0, 2), Index2::new(1, 1));

        // when
        let selection_columns = selection.selected_columns_in_row(5, 0);

        // then
        assert_eq!(selection_columns, Some((2, 5)));

        // when
        let selection = Selection::new(Index2::new(1, 2), Index2::new(1, 1));
        let selection_columns = selection.selected_columns_in_row(5, 0);

        // then
        assert_eq!(selection_columns, None);
    }
}

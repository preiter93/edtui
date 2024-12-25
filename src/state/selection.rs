use std::cmp::Ordering;

use jagged::index::RowIndex;

use crate::{Index2, Lines};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub start: Index2,
    pub end: Index2,
    pub line_mode: bool,
}

impl Selection {
    #[must_use]
    pub fn new(start: Index2, end: Index2) -> Self {
        Self {
            start,
            end,
            line_mode: false,
        }
    }

    pub fn line_mode(mut self) -> Self {
        self.line_mode = true;
        self
    }

    #[must_use]
    pub fn contains(&self, pos: &Index2) -> bool {
        if self.line_mode {
            return self.contains_row(pos.row);
        }

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

    /// Copies a selection from `Lines`.
    #[must_use]
    pub fn copy_from(&self, lines: &Lines) -> Lines {
        if self.line_mode {
            let mut st = self.start();
            let mut en = self.end();
            st.col = 0;
            en.col = lines.last_col_index(en.row);

            let mut lines = lines.copy_range(st..=en);
            lines.insert(RowIndex::new(0), vec![]);

            return lines;
        }

        lines.copy_range(self.start()..=self.end())
    }

    /// Extracts a selection from `Lines`.
    #[must_use]
    pub fn extract_from(&self, lines: &mut Lines) -> Lines {
        if self.line_mode {
            let st = Index2::new(self.start().row, 0);
            let en = Index2::new(self.end().row, lines.last_col_index(self.end().row));

            let mut lines = lines.extract(st..=en);
            lines.insert(RowIndex::new(0), vec![]);

            return lines;
        }

        lines.extract(self.start()..=self.end())
    }

    /// Returns the start and end column of the selection in the given row.
    /// If the selection does not intersect with the row, the function returns None.
    #[must_use]
    pub(crate) fn get_selected_columns_in_row(
        &self,
        row_index: usize,
        row_len: usize,
    ) -> Option<(usize, usize)> {
        let (start, end) = (self.start(), self.end());

        let start_col = match start.row.cmp(&row_index) {
            Ordering::Less => 0,
            Ordering::Greater => return None,
            Ordering::Equal => start.col.min(row_len),
        };

        let end_col = match end.row.cmp(&row_index) {
            Ordering::Less => return None,
            Ordering::Greater => row_len,
            Ordering::Equal => end.col.min(row_len),
        };

        Some((start_col, end_col))
    }
}

/// Set the selections end positions
pub(crate) fn set_selection(selection: &mut Option<Selection>, index: Index2) {
    if let Some(selection) = selection {
        selection.end = index;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_data() -> Lines {
        Lines::from("Hello\nWorld")
    }

    #[test]
    fn test_copy_from() {
        let data = test_data();
        let selection = Selection::new(Index2::new(0, 3), Index2::new(1, 1));

        assert_eq!(selection.copy_from(&data), Lines::from("lo\nWo"));
    }

    #[test]
    fn test_copy_from_out_of_bounds() {
        let data = test_data();
        let selection = Selection::new(Index2::new(0, 5), Index2::new(1, 1));

        assert_eq!(selection.copy_from(&data), Lines::from("\nWo"));
    }

    #[test]
    fn test_selection_columns_in_row() {
        // given
        let selection = Selection::new(Index2::new(0, 2), Index2::new(1, 1));

        // when
        let selection_columns = selection.get_selected_columns_in_row(0, 5);

        // then
        assert_eq!(selection_columns, Some((2, 5)));

        // when
        let selection = Selection::new(Index2::new(1, 2), Index2::new(1, 1));
        let selection_columns = selection.get_selected_columns_in_row(0, 5);

        // then
        assert_eq!(selection_columns, None);
    }
}

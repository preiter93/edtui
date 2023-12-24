use crate::Index2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub start: Index2,
    pub end: Index2,
}

impl Selection {
    pub fn new(start: Index2, end: Index2) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub fn within(&self, pos: &Index2) -> bool {
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
    pub fn start(&self) -> Index2 {
        if self.reverse() {
            return self.end;
        }
        self.start
    }

    #[must_use]
    pub fn end(&self) -> Index2 {
        if self.reverse() {
            return self.start;
        }
        self.end
    }

    #[must_use]
    fn reverse(&self) -> bool {
        self.start.row > self.end.row
            || self.start.row == self.end.row && self.start.col > self.end.col
    }
}

use super::position::Position;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub start: Position,
    pub end: Position,
}

impl Selection {
    #[must_use]
    pub fn within(&self, pos: &Position) -> bool {
        let (start, end) = if self.start < self.end {
            (&self.start, &self.end)
        } else {
            (&self.end, &self.start)
        };
        let (start_line, start_column) = (start.line, start.column);
        let (end_line, end_column) = (end.line, end.column);

        match (pos.line, pos.column) {
            (line, _) if line > start_line && line < end_line => true,
            (line, column) if line > start_line && line == end_line => column <= end_column,
            (line, column) if line == start_line && line < end_line => column >= start_column,
            (line, column) if line == start_line && line == end_line => {
                column <= end_column && column >= start_column
            }
            _ => false,
        }
    }

    #[must_use]
    pub fn start(&self) -> Position {
        if self.reverse() {
            return self.end;
        }
        self.start
    }

    #[must_use]
    pub fn end(&self) -> Position {
        if self.reverse() {
            return self.start;
        }
        self.end
    }

    #[must_use]
    fn reverse(&self) -> bool {
        self.start.line > self.end.line
            || self.start.line == self.end.line && self.start.column > self.end.column
    }
}

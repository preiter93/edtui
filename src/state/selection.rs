use super::position::Position;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub start: Position,
    pub end: Position,
}

impl Selection {
    pub fn within(&self, pos: &Position) -> bool {
        let (start_line, start_column) = (self.start.line, self.start.column);
        let (end_line, end_column) = (self.end.line, self.end.column);

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

    pub fn start(&self) -> Position {
        if self.reverse() {
            return self.end.clone();
        }
        self.start.clone()
    }

    pub fn end(&self) -> Position {
        if self.reverse() {
            return self.start.clone();
        }
        self.end.clone()
    }

    fn reverse(&self) -> bool {
        self.start.line > self.end.line
            || self.start.line == self.end.line && self.start.column > self.end.column
    }
}

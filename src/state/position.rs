use jagged::Index2;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    #[must_use]
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    #[must_use]
    pub fn as_index(&self) -> Index2 {
        Index2::new(self.line, self.column)
    }
}
impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.line.cmp(&other.line) {
            std::cmp::Ordering::Less => Some(std::cmp::Ordering::Less),
            std::cmp::Ordering::Greater => Some(std::cmp::Ordering::Greater),
            std::cmp::Ordering::Equal => self.column.partial_cmp(&other.column),
        }
    }
}

impl From<Position> for Index2 {
    fn from(val: Position) -> Self {
        Index2::new(val.line, val.column)
    }
}
impl From<Index2> for Position {
    fn from(val: Index2) -> Self {
        Self::new(val.row, val.col)
    }
}

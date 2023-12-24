use jagged::{index::RowIndex, Index2};

use crate::{state::selection::Selection, EditorState, Lines};

/// Checks whether an index is the last one in a row.
/// Returns true if the rows index is out of bounds.
pub(crate) fn is_last_index(lines: &Lines, index: Index2) -> bool {
    index.col >= lines.len_col(index.row).saturating_sub(1)
}

/// Set the selections end positions
pub(crate) fn set_selection(selection: &mut Option<Selection>, end: Index2) {
    if let Some(start) = selection.as_ref().map(|x| x.start) {
        *selection = Some(Selection::new(start, end.into()));
    }
}

/// Skip whitespaces moving to the right. Stop at the end of the line.
pub(crate) fn skip_whitespace(lines: &Lines, index: &mut Index2) {
    if let Some(line) = lines.get(RowIndex::new(index.row)) {
        for (i, &ch) in line.iter().enumerate().skip(index.col) {
            if !ch.is_ascii_whitespace() {
                index.col = i;
                break;
            }
        }
    }
}

/// Skip whitespaces moving to the left. Stop at the start of the line.
pub(crate) fn skip_whitespace_rev(lines: &Lines, index: &mut Index2) {
    if let Some(line) = lines.get(RowIndex::new(index.row)) {
        let skip = line.len().saturating_sub(index.col + 1);
        for &ch in line.iter().rev().skip(skip) {
            if !ch.is_ascii_whitespace() {
                break;
            }
            index.col = index.col.saturating_sub(1);
        }
    }
}

/// Get the number of columns in the current line.
#[must_use]
pub(crate) fn len_col(state: &EditorState) -> usize {
    state.lines.len_col(state.cursor.line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_whitespace() {
        let lines = Lines::from("  World!");
        let mut index = Index2::new(0, 0);

        skip_whitespace(&lines, &mut index);
        assert_eq!(index.col, 2);

        skip_whitespace(&lines, &mut index);
        assert_eq!(index.col, 2);
    }

    #[test]
    fn test_skip_whitespace_rev() {
        let lines = Lines::from("  x World!");
        let mut index = Index2::new(0, 3);

        skip_whitespace_rev(&lines, &mut index);
        assert_eq!(index.col, 2);

        skip_whitespace_rev(&lines, &mut index);
        assert_eq!(index.col, 2);

        index.col = 1;
        skip_whitespace_rev(&lines, &mut index);
        assert_eq!(index.col, 0);
    }
}

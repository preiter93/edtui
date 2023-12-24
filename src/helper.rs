use jagged::index::RowIndex;

use crate::{state::selection::Selection, EditorMode, EditorState, Index2, Lines};

/// Returns the maximum permissible column value. In normal or visual
/// mode the limit is len() - 1, in insert mode the limit is len().
pub(crate) fn max_col(lines: &Lines, index: &Index2, mode: EditorMode) -> usize {
    if mode == EditorMode::Insert {
        lines.len_col(index.row)
    } else {
        lines.len_col(index.row).saturating_sub(1)
    }
}

/// Returns the maximum permissible column value. In normal or visual
/// mode the limit is len() - 1, in insert mode the limit is len().
pub(crate) fn max_row(state: &EditorState) -> usize {
    if state.mode == EditorMode::Insert {
        state.lines.len()
    } else {
        state.lines.len().saturating_sub(1)
    }
}

/// Clamps the column of the cursor if the cursor is out of bounds.
/// In normal or visual mode, clamps on col = len() - 1, in insert
/// mode on col = len().
pub(crate) fn clamp_column(state: &mut EditorState) {
    let max_col = max_col(&state.lines, &state.cursor, state.mode);
    state.cursor.col = state.cursor.col.min(max_col);
}

/// Set the selections end positions
pub(crate) fn set_selection(selection: &mut Option<Selection>, end: Index2) {
    if let Some(start) = selection.as_ref().map(|x| x.start) {
        *selection = Some(Selection::new(start, end));
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
    state.lines.len_col(state.cursor.row)
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

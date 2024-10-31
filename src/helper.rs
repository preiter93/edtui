use jagged::index::RowIndex;
use ratatui::{layout::Rect, text::Span};

use crate::{EditorMode, EditorState, Index2, Lines};

/// Inserts a character into the lines data at the given `index`.
pub(crate) fn insert_char(lines: &mut Lines, index: &mut Index2, ch: char, skip_move: bool) {
    if lines.len() == index.row {
        lines.push(Vec::new());
    }
    if ch == '\n' {
        line_break(lines, index);
    } else {
        let Some(len_col) = lines.len_col(index.row) else {
            return;
        };
        if index.col > len_col {
            index.col = len_col.saturating_sub(1);
        }
        lines.insert(*index, ch);
        if !skip_move {
            index.col += 1;
        }
    }
}

/// Inserts a string into the lines data at the given `index`.
pub(crate) fn insert_str(lines: &mut Lines, index: &mut Index2, text: &str) {
    for (i, ch) in text.chars().enumerate() {
        let is_last = i == text.len().saturating_sub(1);
        insert_char(lines, index, ch, is_last);
    }
}

/// Appends a string into the lines data next to a given `index`.
pub(crate) fn append_str(lines: &mut Lines, index: &mut Index2, text: &str) {
    if !lines.is_empty() && lines.len_col(index.row).unwrap_or_default() > 0 {
        index.col += 1;
    }
    for ch in text.chars() {
        insert_char(lines, index, ch, false);
    }
    index.col = index.col.saturating_sub(1);
}

/// Inserts a line break at a given index. Forces a splitting of lines if
/// the index is in the middle of a line.
pub(crate) fn line_break(lines: &mut Lines, index: &mut Index2) {
    if index.col == 0 {
        lines.insert(RowIndex::new(index.row), vec![]);
    } else if index.col >= lines.len_col(index.row).unwrap_or_default() {
        if index.row == lines.len() {
            lines.insert(RowIndex::new(index.row), vec![]);
        } else {
            lines.insert(RowIndex::new(index.row + 1), vec![]);
        }
    } else {
        let mut rest = lines.split_off(*index);
        lines.append(&mut rest);
    }
    index.row += 1;
    index.col = 0;
}

/// Returns the maximum permissible column value. In normal mode
/// the limit is `len() - 1`, in visual and insert mode the limit is `len()`.
pub(crate) fn max_col(lines: &Lines, index: &Index2, mode: EditorMode) -> usize {
    if mode == EditorMode::Normal {
        max_col_normal(lines, index)
    } else {
        max_col_insert(lines, index)
    }
}

/// Returns the maximum permissible column value.
pub(crate) fn max_col_normal(lines: &Lines, index: &Index2) -> usize {
    if lines.is_empty() {
        return 0;
    }
    let Some(len_col) = lines.len_col(index.row) else {
        return 0;
    };
    len_col.saturating_sub(1)
}

/// Returns the maximum permissible column value.
pub(crate) fn max_col_insert(lines: &Lines, index: &Index2) -> usize {
    if lines.is_empty() {
        return 0;
    }
    lines.len_col(index.row).unwrap_or_default()
}

/// Returns the maximum permissible column value. In normal or visual
/// mode the limit is `len() - 1`, in insert mode the limit is `len()`.
pub(crate) fn max_row(state: &EditorState) -> usize {
    if state.mode == EditorMode::Insert {
        state.lines.len()
    } else {
        state.lines.len().saturating_sub(1)
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

/// Skip empty lines.
pub(crate) fn skip_empty_lines(lines: &Lines, row_index: &mut usize) {
    for line in lines.iter_row().skip(*row_index) {
        if !line.is_empty() {
            break;
        }
        *row_index += 1;
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
    state.lines.len_col(state.cursor.row).unwrap_or_default()
}

/// Checks whether an index is out of bounds of the `Lines` buffer.
pub(crate) fn is_out_of_bounds(lines: &Lines, index: &Index2) -> bool {
    if index.row >= lines.len() {
        return true;
    }

    if index.col >= lines.len_col_unchecked(index.row) {
        return true;
    }

    false
}

/// Finds the index of the matching (closing or opening) bracket from a given starting point.
#[must_use]
pub(crate) fn find_matching_bracket(lines: &Lines, index: Index2) -> Option<Index2> {
    let &opening_bracket = lines.get(index)?;

    let (closing_bracket, reverse) = match opening_bracket {
        '{' => ('}', false),
        '}' => ('{', true),
        '(' => (')', false),
        ')' => ('(', true),
        '[' => (']', false),
        ']' => ('[', true),
        _ => return None,
    };

    let mut counter = 0;

    let iter: Box<dyn Iterator<Item = (Option<&char>, Index2)>> = if reverse {
        Box::new(lines.iter().from(index).rev().skip(1))
    } else {
        Box::new(lines.iter().from(index).skip(1))
    };

    for (value, index) in iter {
        let Some(&value) = value else { continue };

        if value == opening_bracket {
            counter += 1;
        }

        if value == closing_bracket {
            if counter == 0 {
                return Some(index);
            }
            counter -= 1;
        }
    }

    None
}

/// Determines the unicode width of a char.
pub(crate) fn char_width(ch: char, tab_width: usize) -> usize {
    use unicode_width::UnicodeWidthChar;
    if ch == '\t' {
        return tab_width;
    }
    ch.width().unwrap_or(0)
}

/// Determines the unicode width of chars.
pub(crate) fn chars_width(chars: &[char], tab_width: usize) -> usize {
    chars
        .iter()
        .fold(0, |sum, ch| sum + char_width(*ch, tab_width))
}

/// Determines the unicode width of a span.
pub(crate) fn span_width(s: &Span, tab_width: usize) -> usize {
    use unicode_width::UnicodeWidthStr;
    s.content
        .as_ref()
        .replace('\t', &" ".repeat(tab_width))
        .width()
}

/// Splits span into two at an index. Other than [`str::split_at`], this method
/// does not fail on splits outside of unicode character boundaries.
pub(crate) fn split_str_at<T: AsRef<str>>(s: T, mid: usize) -> (String, String) {
    let mut chars = s.as_ref().chars();
    let first_half: String = chars.by_ref().take(mid).collect();
    let second_half: String = chars.collect();

    (first_half, second_half)
}

pub(crate) fn replace_tabs_in_span(span: &mut Span, tab_width: usize) {
    span.content = span.content.replace('\t', &" ".repeat(tab_width)).into();
}

pub(crate) fn rect_indent_y(rect: Rect, offset: usize) -> Rect {
    Rect {
        y: rect.y.saturating_add(offset as u16),
        height: rect.width.saturating_sub(offset as u16),
        ..rect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_lines() -> Lines {
        Lines::from("Hello World!\n\n123.")
    }

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
    fn test_skip_empty_lines() {
        let lines = test_lines();

        let mut row_index = 0;
        skip_empty_lines(&lines, &mut row_index);
        assert_eq!(row_index, 0);

        let mut row_index = 1;
        skip_empty_lines(&lines, &mut row_index);
        assert_eq!(row_index, 2);

        let mut row_index = 2;
        skip_empty_lines(&lines, &mut row_index);
        assert_eq!(row_index, 2);
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

    #[test]
    fn test_insert_str() {
        let mut lines = test_lines();
        let mut index = Index2::new(0, 5);

        insert_str(&mut lines, &mut index, ",\n");
        assert_eq!(index, Index2::new(1, 0));
        assert_eq!(lines, Lines::from("Hello,\n World!\n\n123."));
    }

    #[test]
    fn test_insert_char() {
        let mut lines = test_lines();
        let mut index = Index2::new(0, 5);

        insert_char(&mut lines, &mut index, '?', false);
        assert_eq!(index, Index2::new(0, 6));
        assert_eq!(lines, Lines::from("Hello? World!\n\n123."));
    }

    #[test]
    fn test_insert_char_out_of_bounds() {
        let mut lines = test_lines();
        let mut index = Index2::new(99, 0);

        insert_char(&mut lines, &mut index, '?', false);
        assert_eq!(index, Index2::new(99, 0));
        assert_eq!(lines, Lines::from("Hello World!\n\n123."));
    }

    #[test]
    fn test_append_str() {
        let mut lines = test_lines();
        let mut index = Index2::new(0, 5);

        append_str(&mut lines, &mut index, ",\n");
        assert_eq!(index, Index2::new(1, 0));
        assert_eq!(lines, Lines::from("Hello ,\nWorld!\n\n123."));

        let mut lines = test_lines();
        let mut index = Index2::new(1, 0);
        append_str(&mut lines, &mut index, "abc");
        assert_eq!(index, Index2::new(1, 2));
        assert_eq!(lines, Lines::from("Hello World!\nabc\n123."));
    }

    #[test]
    fn test_find_matching_bracket() {
        let cursor = Index2::new(0, 0);
        let lines = Lines::from("{ab\n{{}}c}d");

        let closing_bracket = find_matching_bracket(&lines, cursor);
        assert_eq!(closing_bracket, Some(Index2::new(1, 5)));

        let cursor = Index2::new(1, 5);
        let closing_bracket = find_matching_bracket(&lines, cursor);
        assert_eq!(closing_bracket, Some(Index2::new(0, 0)));
    }
}

use jagged::index::RowIndex;

use super::Execute;
use crate::{
    actions::motion::{find_char_forward, CharacterClass},
    clipboard::ClipboardTrait,
    helper::{
        is_out_of_bounds, max_col_insert, max_col_normal, skip_whitespace, skip_whitespace_rev,
    },
    state::selection::Selection,
    EditorState, Index2, Lines,
};

/// Deletes a character at the current cursor position. Does not
/// move the cursor position unless it is at the end of the line
/// Intended to be called in normal mode.
#[derive(Clone, Debug, Copy)]
pub struct RemoveChar(pub usize);

impl Execute for RemoveChar {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        state.clamp_column();
        for _ in 0..self.0 {
            let lines = &mut state.lines;
            let index = &mut state.cursor;

            if is_out_of_bounds(lines, index) {
                return;
            }

            let _ = lines.remove(*index);
            index.col = index.col.min(
                lines
                    .len_col(index.row)
                    .unwrap_or_default()
                    .saturating_sub(1),
            );
        }
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

/// Replaces the character under the cursor with a given character.
/// Intended to be called in normal mode.
#[derive(Clone, Debug, Copy)]
pub struct ReplaceChar(pub char);

impl Execute for ReplaceChar {
    fn execute(&mut self, state: &mut EditorState) {
        let index = state.cursor;
        if is_out_of_bounds(&state.lines, &index) {
            return;
        }
        state.capture();
        if let Some(ch) = state.lines.get_mut(index) {
            *ch = self.0;
        };
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

/// Deletes a character to the left of the current cursor. Deletes
/// the line break if the the cursor is in column zero.
/// Intended to be called in insert mode.
#[derive(Clone, Debug, Copy)]
pub struct DeleteChar(pub usize);

impl Execute for DeleteChar {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        for _ in 0..self.0 {
            delete_char(&mut state.lines, &mut state.cursor);
        }
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

fn delete_char(lines: &mut Lines, index: &mut Index2) {
    fn move_left(lines: &Lines, index: &mut Index2) {
        if index.col > 0 {
            index.col -= 1;
        } else if index.row > 0 {
            index.row -= 1;
            index.col = lines.len_col(index.row).unwrap_or_default();
        }
    }

    let len_col = lines.len_col(index.row).unwrap_or_default();
    if len_col == 0 && index.row == 0 {
        return;
    }

    if index.col > len_col {
        index.col = len_col;
    }

    if index.col == 0 {
        let mut rest = lines.split_off(*index);
        move_left(lines, index);
        lines.merge(&mut rest);
    } else {
        let max_col = max_col_insert(lines, index);
        index.col = index.col.min(max_col);
        move_left(lines, index);
        let _ = lines.remove(*index);
    }
}

/// Deletes the character at the current cursor position.
/// If at the end of a line, deletes the newline character.
/// Intended to be called in insert mode.
#[derive(Clone, Debug, Copy)]
pub struct DeleteCharForward(pub usize);

impl Execute for DeleteCharForward {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        state.clamp_column();
        for _ in 0..self.0 {
            delete_char_forward(&mut state.lines, &mut state.cursor);
        }
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

fn delete_char_forward(lines: &mut Lines, index: &mut Index2) {
    let Some(row) = lines.get(RowIndex::new(index.row)) else {
        return;
    };

    let row_len = row.len();

    // If cursor is at or past the end of the line, delete the newline
    if index.col >= row_len {
        if index.row + 1 >= lines.len() {
            return;
        }

        lines.join_lines(index.row);
        return;
    }

    let _ = lines.remove(*index);
}

/// Deletes from cursor to the end of the current word (Emacs Alt+d).
#[derive(Clone, Debug, Copy)]
pub struct DeleteWordForward(pub usize);

impl Execute for DeleteWordForward {
    fn execute(&mut self, state: &mut EditorState) {
        if state.lines.is_empty() {
            return;
        }
        state.capture();
        for _ in 0..self.0 {
            delete_word_forward(state);
        }
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

fn delete_motion_forward<F>(
    state: &mut EditorState,
    mut should_continue: F,
    skip_trailing_whitespace: bool,
) where
    F: FnMut(Option<&char>, Option<&char>) -> bool,
{
    let mut start = state.cursor;
    let len_col = state.lines.len_col(start.row).unwrap_or_default();

    // On an empty line there is nothing to delete
    if len_col == 0 {
        if start.row + 1 < state.lines.len() {
            state.lines.join_lines(start.row);
        }
        return;
    }

    // Clamp the cursor when it sits past the end of the line
    start.col = start.col.min(max_col_normal(&state.lines, &start));
    state.cursor = start;

    let start_char = state.lines.get(start);
    let mut end = start;

    for (ch, idx) in state.lines.iter().from(start) {
        // Stop at the end of the current line
        // A forward word delete must not cross into the next line.
        if idx.row != start.row || !should_continue(start_char, ch) {
            break;
        }
        end = idx;
    }
    end.col += 1;

    if skip_trailing_whitespace {
        skip_whitespace(&state.lines, &mut end);
    }
    delete_range(&mut state.lines, start, end, &mut state.clip);

    state.cursor.col = state
        .cursor
        .col
        .min(max_col_normal(&state.lines, &state.cursor));
}

/// Deletes from cursor forward to the next word boundary (Vim `dw`).
fn delete_word_forward(state: &mut EditorState) {
    delete_motion_forward(
        state,
        |a, b| CharacterClass::from(a) == CharacterClass::from(b),
        true,
    );
}

/// Deletes from cursor forward to the next WORD boundary (Vim `dW`).
/// A WORD is any sequence of non-whitespace characters.
#[derive(Clone, Debug, Copy)]
pub struct DeleteBigWordForward(pub usize);

impl Execute for DeleteBigWordForward {
    fn execute(&mut self, state: &mut EditorState) {
        if state.lines.is_empty() {
            return;
        }
        state.capture();
        for _ in 0..self.0 {
            delete_big_word_forward(state);
        }
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

fn delete_big_word_forward(state: &mut EditorState) {
    delete_motion_forward(
        state,
        |a, b| {
            let a_ws = CharacterClass::from(a) == CharacterClass::Whitespace;
            let b_ws = CharacterClass::from(b) == CharacterClass::Whitespace;
            a_ws == b_ws
        },
        true,
    );
}

/// Deletes from the cursor to the end of the current word, without consuming
/// trailing whitespace. This is the `cw` primitive, which in Vim behaves like
/// `ce` rather than `dw`.
#[derive(Clone, Debug, Copy)]
pub struct DeleteWordEnd(pub usize);

impl Execute for DeleteWordEnd {
    fn execute(&mut self, state: &mut EditorState) {
        if state.lines.is_empty() {
            return;
        }
        state.capture();
        for _ in 0..self.0 {
            delete_word_end(state);
        }
    }
}

fn delete_word_end(state: &mut EditorState) {
    delete_motion_forward(
        state,
        |a, b| CharacterClass::from(a) == CharacterClass::from(b),
        false,
    );
}

/// Deletes from the cursor to the end of the current WORD, without consuming
/// trailing whitespace. This is the `cW` primitive, which in Vim behaves like
/// `cE` rather than `dW`. A WORD is any sequence of non-whitespace characters.
#[derive(Clone, Debug, Copy)]
pub struct DeleteBigWordEnd(pub usize);

impl Execute for DeleteBigWordEnd {
    fn execute(&mut self, state: &mut EditorState) {
        if state.lines.is_empty() {
            return;
        }
        state.capture();
        for _ in 0..self.0 {
            delete_big_word_end(state);
        }
    }
}

fn delete_big_word_end(state: &mut EditorState) {
    delete_motion_forward(
        state,
        |a, b| {
            let a_ws = CharacterClass::from(a) == CharacterClass::Whitespace;
            let b_ws = CharacterClass::from(b) == CharacterClass::Whitespace;
            a_ws == b_ws
        },
        false,
    );
}

/// Deletes from cursor backward to start of previous word (Emacs Alt+Backspace).
#[derive(Clone, Debug, Copy)]
pub struct DeleteWordBackward(pub usize);

impl Execute for DeleteWordBackward {
    fn execute(&mut self, state: &mut EditorState) {
        if state.lines.is_empty() {
            return;
        }
        state.capture();
        for _ in 0..self.0 {
            delete_word_backward(state);
        }
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

fn delete_word_backward(state: &mut EditorState) {
    let end = state.cursor;

    if end.row == 0 && end.col == 0 {
        return;
    }

    if end.col == 0 {
        state.cursor.row -= 1;
        state.cursor.col = state.lines.len_col(state.cursor.row).unwrap_or(0);
        state.lines.join_lines(state.cursor.row);
        return;
    }

    let mut start = Index2::new(end.row, end.col.saturating_sub(1));
    skip_whitespace_rev(&state.lines, &mut start);
    let start_class = CharacterClass::from(state.lines.get(start));

    for (ch, idx) in state.lines.iter().from(start).rev() {
        if idx.col == 0 {
            start = idx;
            break;
        }
        if CharacterClass::from(ch) != start_class {
            break;
        }
        start = idx;
    }

    delete_range(&mut state.lines, start, end, &mut state.clip);
    state.cursor = start;
}

fn delete_range(
    lines: &mut Lines,
    start: Index2,
    end: Index2,
    clip: &mut crate::clipboard::Clipboard,
) {
    if start.row != end.row || start.col >= end.col {
        return;
    }

    let Some(row) = lines.get_mut(RowIndex::new(start.row)) else {
        return;
    };

    let end_col = end.col.min(row.len());
    let start_col = start.col.min(end_col);

    let deleted: String = row.drain(start_col..end_col).collect();
    clip.set_text(deleted);
}

/// Deletes the current line.
#[derive(Clone, Debug, Copy)]
pub struct DeleteLine(pub usize);

impl Execute for DeleteLine {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        for _ in 0..self.0 {
            if state.cursor.row >= state.lines.len() {
                break;
            }
            let row_index = RowIndex::new(state.cursor.row);
            let deleted_line = state.lines.remove(row_index).iter().collect::<String>();
            state.clip.set_text(String::from('\n') + &deleted_line);
            state.cursor.col = 0;
            state.cursor.row = state.cursor.row.min(state.lines.len().saturating_sub(1));
        }
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

/// Deletes from the current cursor position to the first non-whitespace character of the line
#[derive(Clone, Debug, Copy)]
pub struct DeleteToFirstCharOfLine;

impl Execute for DeleteToFirstCharOfLine {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();

        let row_index = RowIndex::new(state.cursor.row);
        let Some(row) = state.lines.get_mut(row_index) else {
            return;
        };

        let col = state.cursor.col;

        let first_char = row
            .iter()
            .position(|c| !c.is_whitespace())
            .unwrap_or(row.len());

        let anchor = if col <= first_char { 0 } else { first_char };

        if anchor < col && col <= row.len() {
            let deleted = row.drain(anchor..col).collect();
            state.clip.set_text(deleted);
        }

        state.cursor.col = anchor;
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

/// Deletes from the current cursor position to the end of the line
#[derive(Clone, Debug, Copy)]
pub struct DeleteToEndOfLine;

impl Execute for DeleteToEndOfLine {
    fn execute(&mut self, state: &mut EditorState) {
        if is_out_of_bounds(&state.lines, &state.cursor) {
            return;
        }
        state.capture();
        let Some(row) = state.lines.get_mut(RowIndex::new(state.cursor.row)) else {
            return;
        };
        let deleted_chars = row.drain(state.cursor.col..);
        state.cursor.col = state.cursor.col.saturating_sub(1);
        state.clip.set_text(deleted_chars.collect());
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

/// Deletes from the cursor up to and including the next occurrence of a
/// character on the current line (Vim `df<char>`). Does nothing if the
/// character is not found.
///
/// The target is `None` until the key handler supplies the next keystroke via
/// [`Execute::char_arg`].
#[derive(Clone, Debug, Copy)]
pub struct DeleteFindForward(pub Option<char>);

impl Execute for DeleteFindForward {
    fn execute(&mut self, state: &mut EditorState) {
        let Some(target_char) = self.0 else {
            return;
        };
        let Some(target) = find_char_forward(state, target_char) else {
            return;
        };
        state.capture();
        let Some(row) = state.lines.get_mut(RowIndex::new(state.cursor.row)) else {
            return;
        };
        let deleted = row.drain(state.cursor.col..=target).collect();
        state.clip.set_text(deleted);
        state.clamp_column();
    }

    fn is_repeatable(&self) -> bool {
        true
    }

    fn char_arg(&mut self) -> Option<&mut Option<char>> {
        Some(&mut self.0)
    }
}

/// Deletes from the cursor up to (but not including) the next occurrence of a
/// character on the current line (Vim `dt<char>`). Does nothing if the
/// character is not found.
///
/// The target is `None` until the key handler supplies the next keystroke via
/// [`Execute::char_arg`].
#[derive(Clone, Debug, Copy)]
pub struct DeleteTillForward(pub Option<char>);

impl Execute for DeleteTillForward {
    fn execute(&mut self, state: &mut EditorState) {
        let Some(target_char) = self.0 else {
            return;
        };
        let Some(target) = find_char_forward(state, target_char) else {
            return;
        };
        state.capture();
        let Some(row) = state.lines.get_mut(RowIndex::new(state.cursor.row)) else {
            return;
        };
        let deleted = row.drain(state.cursor.col..target).collect();
        state.clip.set_text(deleted);
        state.clamp_column();
    }

    fn is_repeatable(&self) -> bool {
        true
    }

    fn char_arg(&mut self) -> Option<&mut Option<char>> {
        Some(&mut self.0)
    }
}

/// Deletes the current selection.
#[derive(Clone, Debug)]
pub struct DeleteSelection;

impl Execute for DeleteSelection {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(selection) = state.selection.take() {
            state.capture();
            let drained = delete_selection(state, &selection);
            state.clip.set_text(drained.into());
        }
        state.selection = None;
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

pub(crate) fn delete_selection(state: &mut EditorState, selection: &Selection) -> Lines {
    state.cursor = selection.start();
    state.clamp_column();
    selection.extract_from(&mut state.lines)
}

/// Joins line below to the current line.
#[derive(Clone, Debug, Copy)]
pub struct JoinLineWithLineBelow;

impl Execute for JoinLineWithLineBelow {
    fn execute(&mut self, state: &mut EditorState) {
        if state.cursor.row + 1 >= state.lines.len() {
            return;
        }
        state.capture();
        state.lines.join_lines(state.cursor.row);
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::state::selection::Selection;
    use crate::EditorMode;
    use crate::Index2;
    use crate::Lines;

    use super::*;
    fn test_state() -> EditorState {
        EditorState::new(Lines::from("Hello World!\n\n123."))
    }

    #[test]
    fn test_remove_char() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 4);
        RemoveChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hell World!\n\n123."));

        state.cursor = Index2::new(0, 10);
        RemoveChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 9));
        assert_eq!(state.lines, Lines::from("Hell World\n\n123."));
    }

    #[test]
    fn test_replace_char() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 4);
        ReplaceChar('x').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hellx World!\n\n123."));

        // do nothing on empty line
        state.cursor = Index2::new(1, 0);
        ReplaceChar('x').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
        assert_eq!(state.lines, Lines::from("Hellx World!\n\n123."));

        // do nothing if out of bounds
        state.cursor = Index2::new(99, 0);
        ReplaceChar('x').execute(&mut state);
        assert_eq!(state.cursor, Index2::new(99, 0));
        assert_eq!(state.lines, Lines::from("Hellx World!\n\n123."));
    }

    #[test]
    fn test_delete_char() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 5);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hell World!\n\n123."));

        state.cursor = Index2::new(0, 11);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));
        assert_eq!(state.lines, Lines::from("Hell World\n\n123."));
    }

    #[test]
    fn test_delete_char_empty_line() {
        let mut state = test_state();
        state.cursor = Index2::new(1, 99);

        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 12));
        assert_eq!(state.lines, Lines::from("Hello World!\n123."));

        let mut state = EditorState::new(Lines::from("\nb"));
        state.cursor = Index2::new(0, 1);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.lines, Lines::from("\nb"));
    }

    #[test]
    fn test_delete_line() {
        let mut state = test_state();
        state.cursor = Index2::new(2, 3);

        DeleteLine(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
        assert_eq!(state.lines, Lines::from("Hello World!\n"));

        DeleteLine(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from("Hello World!"));

        DeleteLine(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from(""));
    }

    #[test]
    fn test_delete_to_first_char_of_line() {
        let mut state = EditorState::new(Lines::from("  Hello World!"));
        state.cursor = Index2::new(0, 4);

        DeleteToFirstCharOfLine.execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 2));
        assert_eq!(state.lines, Lines::from("  llo World!"));

        state.cursor = Index2::new(0, 2);
        DeleteToFirstCharOfLine.execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from("llo World!"));
    }

    #[test]
    fn test_delete_to_end_of_line() {
        let mut state = test_state();
        state.cursor = Index2::new(0, 3);

        DeleteToEndOfLine.execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 2));
        assert_eq!(state.lines, Lines::from("Hel\n\n123."));
    }

    #[test]
    fn test_delete_selection() {
        let mut state = test_state();
        let st = Index2::new(0, 1);
        let en = Index2::new(2, 0);
        state.selection = Some(Selection::new(st, en));

        DeleteSelection.execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.lines, Lines::from("H23."));
    }

    #[test]
    fn test_delete_selection_out_of_bounds() {
        let mut state = EditorState::new(Lines::from("123.\nHello World!\n456."));
        let st = Index2::new(0, 5);
        let en = Index2::new(2, 10);
        state.selection = Some(Selection::new(st, en));

        DeleteSelection.execute(&mut state);
        // assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.lines, Lines::from("123."));
    }

    #[test]
    fn test_delete_char_forward() {
        let mut state = EditorState::new(Lines::from("Hello World!\nNext line"));
        state.mode = EditorMode::Insert;

        // Delete character 'H'
        state.cursor = Index2::new(0, 0);
        DeleteCharForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from("ello World!\nNext line"));

        // Delete character 'e'
        state.cursor = Index2::new(0, 0);
        DeleteCharForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from("llo World!\nNext line"));

        // Delete character at end of line (newline)
        state.cursor = Index2::new(0, 10);
        DeleteCharForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));
        assert_eq!(state.lines, Lines::from("llo World!Next line"));
    }

    #[test]
    fn test_delete_char_forward_at_end() {
        let mut state = EditorState::new(Lines::from("Hello\nWorld"));
        state.mode = EditorMode::Insert;

        // Cursor at end of first line should delete newline
        state.cursor = Index2::new(0, 5);
        DeleteCharForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 5));
        assert_eq!(state.lines, Lines::from("HelloWorld"));

        // Cursor at end of last line should do nothing
        state.cursor = Index2::new(0, 10);
        DeleteCharForward(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));
        assert_eq!(state.lines, Lines::from("HelloWorld"));
    }

    #[test]
    fn test_delete_word_forward() {
        let mut state = EditorState::new(Lines::from("Hello World Test"));
        state.mode = EditorMode::Insert;
        state.cursor = Index2::new(0, 0);

        DeleteWordForward(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), "World Test");
        assert_eq!(state.cursor, Index2::new(0, 0));

        DeleteWordForward(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), "Test");
    }

    #[test]
    fn test_delete_word_forward_mid_word() {
        let mut state = EditorState::new(Lines::from("Hello World"));
        state.mode = EditorMode::Insert;
        state.cursor = Index2::new(0, 2);

        DeleteWordForward(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), "HeWorld");
    }

    #[test]
    fn test_delete_word_forward_last_character() {
        let mut state = EditorState::new(Lines::from("Hello World"));
        state.mode = EditorMode::Insert;
        state.cursor = Index2::new(0, 10);

        DeleteWordForward(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), "Hello Worl");
        assert_eq!(state.cursor, Index2::new(0, 9));

        state.cursor = Index2::new(0, 10);
        DeleteWordForward(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), "Hello Wor");
        assert_eq!(state.cursor, Index2::new(0, 8));
    }

    #[test]
    fn test_delete_word_forward_does_not_cross_line() {
        // Single char on the first line whose class matches the next line's
        // first char must still be deleted (must not bleed into line below).
        let mut state = EditorState::new(Lines::from("a\nHello"));
        state.mode = EditorMode::Insert;
        state.cursor = Index2::new(0, 0);

        DeleteWordForward(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), "\nHello");

        // Same for a whitespace-delimited WORD delete.
        let mut state = EditorState::new(Lines::from("a\nHello"));
        state.mode = EditorMode::Insert;
        state.cursor = Index2::new(0, 0);

        DeleteBigWordForward(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), "\nHello");
    }

    #[test]
    fn test_delete_big_word_forward() {
        let mut state = EditorState::new(Lines::from("foo.bar baz"));
        state.cursor = Index2::new(0, 0);

        DeleteBigWordForward(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), "baz");
        assert_eq!(state.cursor, Index2::new(0, 0));
    }

    #[test]
    fn test_delete_big_word_forward_mid_word() {
        let mut state = EditorState::new(Lines::from("foo.bar baz"));
        state.cursor = Index2::new(0, 3);

        DeleteBigWordForward(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), "foobaz");
    }

    #[test]
    fn test_delete_word_end_keeps_trailing_whitespace() {
        let mut state = EditorState::new(Lines::from("Hello World"));
        state.cursor = Index2::new(0, 0);

        DeleteWordEnd(1).execute(&mut state);
        // Unlike `dw`, the trailing whitespace is preserved (Vim `cw` == `ce`).
        assert_eq!(state.lines.to_string(), " World");
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.mode, EditorMode::Normal);
    }

    #[test]
    fn test_delete_big_word_vs_small_word() {
        let mut small = EditorState::new(Lines::from("foo.bar baz"));
        small.cursor = Index2::new(0, 0);
        DeleteWordForward(1).execute(&mut small);
        assert_eq!(small.lines.to_string(), ".bar baz");

        let mut big = EditorState::new(Lines::from("foo.bar baz"));
        big.cursor = Index2::new(0, 0);
        DeleteBigWordForward(1).execute(&mut big);
        assert_eq!(big.lines.to_string(), "baz");
    }

    #[test]
    fn test_delete_word_backward() {
        let mut state = EditorState::new(Lines::from("Hello World Test"));
        state.mode = EditorMode::Insert;
        state.cursor = Index2::new(0, 12);

        DeleteWordBackward(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), "Hello Test");
        assert_eq!(state.cursor, Index2::new(0, 6));
    }

    #[test]
    fn test_delete_word_backward_mid_word() {
        // On "o" of World, should only delete "W"
        let mut state = EditorState::new(Lines::from("Hello World"));
        state.mode = EditorMode::Insert;
        state.cursor = Index2::new(0, 7);

        DeleteWordBackward(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), "Hello orld");
        assert_eq!(state.cursor, Index2::new(0, 6));
    }

    #[test]
    fn test_delete_word_backward_at_word_start() {
        // On "W" of World, should delete "Hello " (previous word + whitespace)
        let mut state = EditorState::new(Lines::from("Hello World"));
        state.mode = EditorMode::Insert;
        state.cursor = Index2::new(0, 6);

        DeleteWordBackward(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), "World");
        assert_eq!(state.cursor, Index2::new(0, 0));
    }
}

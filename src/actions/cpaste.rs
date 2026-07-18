use std::cmp::min;

use jagged::{index::RowIndex, Index2};

use crate::{
    clipboard::ClipboardTrait,
    helper::{append_str, insert_str, max_row},
    EditorState,
};

#[cfg(test)]
use crate::EditorMode;

use super::{delete::delete_selection, Execute};

/// Pastes the clipboard contents relative to the cursor.
///
/// With `before` set, characterwise text is inserted in front of the cursor
/// and linewise text (clipboard starting with `\n`) opens a new line *above*
/// the current one (mirroring Vim's `P`). Otherwise the text is pasted after
/// the cursor / on the line below, matching Vim's `p`.
fn paste(state: &mut EditorState, before: bool) {
    let s = state.clip.get_text();
    if s.is_empty() {
        return;
    }

    state.capture();
    state.clamp_column();

    if state.view.single_line {
        let s = s.replace('\n', " ").replace('\r', "");
        if before {
            insert_str(&mut state.lines, &mut state.cursor, &s);
        } else {
            append_str(&mut state.lines, &mut state.cursor, &s);
        }
        return;
    }

    if let Some(stripped) = s.strip_prefix('\n') {
        let row = if before {
            state.cursor.row
        } else {
            min(max_row(state), state.cursor.row + 1)
        };
        state.cursor = Index2::new(row, 0);
        state.lines.insert(RowIndex::new(row), vec![]);
        append_str(&mut state.lines, &mut state.cursor, stripped);
    } else if before {
        insert_str(&mut state.lines, &mut state.cursor, &s);
    } else {
        append_str(&mut state.lines, &mut state.cursor, &s);
    }
}

#[derive(Clone, Debug)]
pub struct Paste;

impl Execute for Paste {
    fn execute(&mut self, state: &mut EditorState) {
        paste(state, false);
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub struct PasteBefore;

impl Execute for PasteBefore {
    fn execute(&mut self, state: &mut EditorState) {
        paste(state, true);
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub struct PasteOverSelection;

impl Execute for PasteOverSelection {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(selection) = state.selection.take() {
            state.capture();
            state.clamp_column();
            let _ = delete_selection(state, &selection);

            // In single-line mode, replace newlines with spaces
            let text = state.clip.get_text();
            let text = if state.view.single_line {
                text.replace('\n', " ").replace('\r', "")
            } else {
                text
            };
            insert_str(&mut state.lines, &mut state.cursor, &text);
        }
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub struct CopySelection;

impl Execute for CopySelection {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(s) = &state.selection {
            state.clip.set_text(s.copy_from(&state.lines).into());
            state.selection = None;
        }
    }
}

#[derive(Clone, Debug)]
pub struct CopyLine;

impl Execute for CopyLine {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(line) = state.lines.get(RowIndex::new(state.cursor.row)) {
            let text = String::from('\n') + &line.iter().collect::<String>();
            state.clip.set_text(text);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::Undo;
    use crate::clipboard::InternalClipboard;
    use crate::state::selection::Selection;
    use crate::Index2;
    use crate::Lines;

    use super::*;
    fn test_state() -> EditorState {
        let mut state = EditorState::new(Lines::from("Hello World!\n\n123."));
        state.set_clipboard(InternalClipboard::default());
        state
    }

    #[test]
    fn test_copy_paste() {
        let mut state = test_state();
        let selection = Selection::new(Index2::new(0, 0), Index2::new(0, 2));
        state.selection = Some(selection);

        CopySelection.execute(&mut state);
        Paste.execute(&mut state);

        assert_eq!(state.cursor, Index2::new(0, 3));
        assert_eq!(state.lines, Lines::from("HHelello World!\n\n123."));
    }

    #[test]
    fn test_paste_before_characterwise() {
        let mut state = test_state();
        state.selection = Some(Selection::new(Index2::new(0, 0), Index2::new(0, 2)));

        CopySelection.execute(&mut state);
        PasteBefore.execute(&mut state);

        // `P` inserts in front of the cursor, landing on the last pasted char.
        assert_eq!(state.cursor, Index2::new(0, 2));
        assert_eq!(state.lines, Lines::from("HelHello World!\n\n123."));
    }

    #[test]
    fn test_paste_before_linewise() {
        let mut state = test_state();
        state.cursor = Index2::new(2, 1);
        state.clip.set_text(String::from("\nnew line"));

        PasteBefore.execute(&mut state);

        // Linewise `P` opens a new line above the current row.
        assert_eq!(state.cursor, Index2::new(2, 7));
        assert_eq!(state.lines, Lines::from("Hello World!\n\nnew line\n123."));
    }

    #[test]
    fn test_paste_with_newline_into_empty_buffer() {
        let mut state = EditorState::default();
        state.set_clipboard(InternalClipboard::default());
        state.clip.set_text("\ntext".to_string());

        Paste.execute(&mut state);

        assert_eq!(state.cursor, Index2::new(0, 3));
        assert_eq!(state.lines, Lines::from("text"));
    }

    #[test]
    fn test_paste_over_selection() {
        let mut state = test_state();
        state.selection = Some(Selection::new(Index2::new(0, 6), Index2::new(0, 10)));
        state.clip.set_text(String::from("Earth"));
        state.mode = EditorMode::Visual;

        PasteOverSelection.execute(&mut state);

        assert_eq!(state.lines, Lines::from("Hello Earth!\n\n123."));
        assert_eq!(state.cursor, Index2::new(0, 10));
        assert_eq!(state.mode, EditorMode::Visual);

        Undo.execute(&mut state);

        assert_eq!(state.lines, Lines::from("Hello World!\n\n123."));
        assert_eq!(state.mode, EditorMode::Visual);
    }
}

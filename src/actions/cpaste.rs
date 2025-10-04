use std::cmp::min;

use jagged::{index::RowIndex, Index2};

use crate::{
    clipboard::ClipboardTrait,
    helper::{append_str, insert_str, max_row},
    EditorMode, EditorState,
};

use super::{delete::delete_selection, Execute, SwitchMode};

#[derive(Clone, Debug)]
pub struct Paste;

impl Execute for Paste {
    fn execute(&mut self, state: &mut EditorState) {
        let s = state.clip.get_text();
        if s.is_empty() {
            return;
        }

        state.capture();
        state.clamp_column();

        let s = if let Some(stripped) = s.strip_prefix('\n') {
            state.cursor = Index2::new(min(max_row(state), state.cursor.row + 1), 0);
            state.lines.insert(RowIndex::new(state.cursor.row), vec![]);
            stripped
        } else {
            state.clamp_column();
            &s
        };

        append_str(&mut state.lines, &mut state.cursor, s);
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
            insert_str(&mut state.lines, &mut state.cursor, &state.clip.get_text());
        }

        SwitchMode(EditorMode::Normal).execute(state);
    }
}

#[derive(Clone, Debug)]
pub struct CopySelection;

impl Execute for CopySelection {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(s) = &state.selection {
            state.clip.set_text(s.copy_from(&state.lines).into());
            state.mode = EditorMode::Normal;
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
        assert_eq!(state.mode, EditorMode::Normal);

        Undo.execute(&mut state);

        assert_eq!(state.lines, Lines::from("Hello World!\n\n123."));
        assert_eq!(state.mode, EditorMode::Normal);
    }
}

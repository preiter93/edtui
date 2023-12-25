use super::Execute;
use crate::{
    clipboard::ClipboardTrait, state::selection::Selection, EditorMode, EditorState, Index2,
};

/// Selects text between specified delimiter characters.
///
/// It searches for the first occurrence of a delimiter character in the text to
/// define the start of the selection, and the next occurrence of any of the delimiter
/// characters to define the end of the selection.
#[derive(Clone, Debug, Copy)]
pub struct SelectBetween(pub char);

impl Execute for SelectBetween {
    fn execute(&mut self, state: &mut EditorState) {
        let cursor = state.cursor;
        let mut start: Option<Index2> = None;
        let mut end: Option<Index2> = None;
        let mut prev = cursor;
        for (value, index) in state.lines.iter().from(cursor) {
            if let Some(&c) = value {
                if c == self.0 {
                    end = Some(prev);
                    break;
                }
            }
            prev = index;
        }
        prev = cursor;
        for (value, index) in state.lines.iter().from(cursor).rev() {
            if let Some(&c) = value {
                if c == self.0 {
                    start = Some(prev);
                    break;
                }
            }
            prev = index;
        }
        if let (Some(start), Some(end)) = (start, end) {
            state.selection = Some(Selection { start, end });
            state.mode = EditorMode::Visual;
        }
    }
}

#[derive(Clone, Debug)]
pub struct CopySelection;

impl Execute for CopySelection {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(s) = &state.selection {
            state.clip.set_text(s.extract(&state.lines).into());
            state.mode = EditorMode::Normal;
            state.selection = None;
        }
    }
}

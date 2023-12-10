use jagged::Index2;

use super::Execute;
use crate::{state::selection::Selection, EditorMode, EditorState};

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
        let mut prev = cursor.as_index();
        for (value, index) in state.lines.iter().from(cursor.as_index()) {
            if let Some(&c) = value {
                if c == self.0 {
                    end = Some(prev);
                    break;
                }
            }
            prev = index;
        }
        prev = cursor.as_index();
        for (value, index) in state.lines.iter().from(cursor.as_index()).rev() {
            if let Some(&c) = value {
                if c == self.0 {
                    start = Some(prev);
                    break;
                }
            }
            prev = index;
        }
        if let (Some(start), Some(end)) = (start, end) {
            state.selection = Some(Selection {
                start: start.into(),
                end: end.into(),
            });
            state.mode = EditorMode::Visual;
        }
    }
}

use crate::{EditorMode, EditorState};

use super::Execute;

/// Appends a single character to the search buffer and triggers
/// the search afterwards.
#[derive(Clone, Debug, Copy)]
pub struct AppendToSearch(pub char);

impl Execute for AppendToSearch {
    fn execute(&mut self, state: &mut EditorState) {
        state.search.push_char(self.0);
        state.search.trigger_search(&state.lines);
    }
}
/// Removes the last character from the search buffer and retriggers
/// the search.
#[derive(Clone, Debug, Copy)]
pub struct DeleteFromSearch;

impl Execute for DeleteFromSearch {
    fn execute(&mut self, state: &mut EditorState) {
        state.search.remove_char();
        state.search.trigger_search(&state.lines);
    }
}

/// Finds the first match of the search behind the last cursor position
/// and sets the cursor to the first match. Switches to normal mode.
#[derive(Clone, Debug)]
pub struct FindFirst;

impl Execute for FindFirst {
    fn execute(&mut self, state: &mut EditorState) {
        state.mode = EditorMode::Normal;
        if let Some(index) = state.search.find_first() {
            state.cursor = *index;
        }
    }
}
/// Finds the next search match and updates the cursor position.
#[derive(Clone, Debug)]
pub struct FindNext;

impl Execute for FindNext {
    fn execute(&mut self, state: &mut EditorState) {
        state.mode = EditorMode::Normal;
        if let Some(index) = state.search.find_next() {
            state.cursor = *index;
        }
    }
}
/// Finds the previous search match and updates the cursor position.
#[derive(Clone, Debug)]
pub struct FindPrevious;

impl Execute for FindPrevious {
    fn execute(&mut self, state: &mut EditorState) {
        state.mode = EditorMode::Normal;
        if let Some(index) = state.search.find_previous() {
            state.cursor = *index;
        }
    }
}

/// Clears the search state and switches into normal mode.
#[derive(Clone, Debug)]
pub struct ClearSearch;

impl Execute for ClearSearch {
    fn execute(&mut self, state: &mut EditorState) {
        state.mode = EditorMode::Normal;
        state.search.clear();
    }
}

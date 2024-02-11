//! Editor actions such as move, insert, delete
pub mod cpaste;
pub mod delete;
pub mod insert;
pub mod motion;
pub mod search;
pub mod select;
use crate::helper::clamp_column;
use crate::state::selection::Selection;
use crate::{EditorMode, EditorState};
use enum_dispatch::enum_dispatch;

pub use self::cpaste::{CopySelection, Paste};
pub use self::delete::{DeleteChar, DeleteLine, DeleteSelection, RemoveChar};
pub use self::insert::{AppendNewline, InsertChar, InsertNewline, LineBreak};
pub use self::motion::{
    MoveBackward, MoveDown, MoveForward, MoveToEnd, MoveToFirst, MoveToStart, MoveUp,
    MoveWordBackward, MoveWordForward,
};
use self::search::StartSearch;
pub use self::search::{
    AppendCharToSearch, FindNext, FindPrevious, RemoveCharFromSearch, StopSearch, TriggerSearch,
};
pub use self::select::SelectBetween;

#[enum_dispatch(Execute)]
#[derive(Clone, Debug)]
pub enum Action {
    SwitchMode(SwitchMode),
    Append(Append),
    MoveForward(MoveForward),
    MoveBackward(MoveBackward),
    MoveUp(MoveUp),
    MoveDown(MoveDown),
    MoveWordForward(MoveWordForward),
    MoveWordBackward(MoveWordBackward),
    MoveToStart(MoveToStart),
    MoveToFirst(MoveToFirst),
    MoveToEnd(MoveToEnd),
    InsertChar(InsertChar),
    LineBreak(LineBreak),
    AppendNewline(AppendNewline),
    InsertNewline(InsertNewline),
    RemoveChar(RemoveChar),
    DeleteChar(DeleteChar),
    DeleteLine(DeleteLine),
    DeleteSelection(DeleteSelection),
    SelectBetween(SelectBetween),
    Undo(Undo),
    Redo(Redo),
    Paste(Paste),
    CopySelection(CopySelection),
    Composed(Composed),
    StartSearch(StartSearch),
    StopSearch(StopSearch),
    TriggerSearch(TriggerSearch),
    FindNext(FindNext),
    FindPrevious(FindPrevious),
    AppendCharToSearch(AppendCharToSearch),
    RemoveCharFromSearch(RemoveCharFromSearch),
}

#[enum_dispatch]
pub trait Execute {
    fn execute(&mut self, state: &mut EditorState);
}

#[derive(Clone, Debug)]
pub struct SwitchMode(pub EditorMode);

impl Execute for SwitchMode {
    fn execute(&mut self, state: &mut EditorState) {
        clamp_column(state);
        match self.0 {
            EditorMode::Normal => {
                state.selection = None;
            }
            EditorMode::Visual => {
                state.selection = Some(Selection::new(state.cursor, state.cursor));
            }
            EditorMode::Insert | EditorMode::Search => {}
        }
        state.mode = self.0;
    }
}

/// Switch to insert mode and move one character forward
#[derive(Clone, Debug)]
pub struct Append;

impl Execute for Append {
    fn execute(&mut self, state: &mut EditorState) {
        SwitchMode(EditorMode::Insert).execute(state);
        MoveForward(1).execute(state);
    }
}

#[derive(Clone, Debug)]
pub struct Undo;

impl Execute for Undo {
    fn execute(&mut self, state: &mut EditorState) {
        state.undo();
    }
}

#[derive(Clone, Debug)]
pub struct Redo;

impl Execute for Redo {
    fn execute(&mut self, state: &mut EditorState) {
        state.redo();
    }
}

/// Executes multiple actions one after the other.
#[derive(Clone, Debug)]
pub struct Composed(pub Vec<Action>);

impl Composed {
    #[must_use]
    pub fn new<A: Into<Action>>(action: A) -> Self {
        Self(vec![action.into()])
    }

    #[must_use]
    pub fn chain<A: Into<Action>>(mut self, action: A) -> Self {
        self.0.push(action.into());
        self
    }
}

impl Execute for Composed {
    fn execute(&mut self, state: &mut EditorState) {
        for action in &mut self.0 {
            action.execute(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::clipboard::InternalClipboard;
    use crate::Index2;
    use crate::Lines;

    use super::*;
    fn test_state() -> EditorState {
        let mut state = EditorState::new(Lines::from("Hello World!\n\n123."));
        state.set_clipboard(InternalClipboard::default());
        state
    }

    #[test]
    fn test_switch_mode() {
        let mut state = test_state();
        assert_eq!(state.mode, EditorMode::Normal);

        SwitchMode(EditorMode::Insert).execute(&mut state);
        assert_eq!(state.mode, EditorMode::Insert);

        SwitchMode(EditorMode::Visual).execute(&mut state);
        assert_eq!(state.mode, EditorMode::Visual);
    }

    #[test]
    fn test_append() {
        let mut state = test_state();

        Append.execute(&mut state);
        assert_eq!(state.mode, EditorMode::Insert);
        assert_eq!(state.cursor, Index2::new(0, 1));

        state.mode = EditorMode::Normal;
        state.cursor = Index2::new(0, 11);
        Append.execute(&mut state);
        assert_eq!(state.mode, EditorMode::Insert);
        assert_eq!(state.cursor, Index2::new(0, 12));
    }
}

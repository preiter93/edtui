pub mod delete;
pub mod insert;
pub mod motion;
pub mod select;
use crate::helper::len_col;
use crate::state::selection::Selection;
use crate::{EditorMode, EditorState};
use enum_dispatch::enum_dispatch;

pub use self::delete::{DeleteChar, DeleteLine, DeleteSelection, RemoveChar};
pub use self::insert::{AppendNewline, InsertChar, InsertNewline, InsertString, LineBreak};
pub use self::motion::{
    MoveBackward, MoveDown, MoveForward, MoveToEnd, MoveToFirst, MoveToStart, MoveUp,
    MoveWordBackward, MoveWordForward,
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
    InsertString(InsertString),
    LineBreak(LineBreak),
    AppendNewline(AppendNewline),
    InsertNewline(InsertNewline),
    DeleteChar(DeleteChar),
    DeleteLine(DeleteLine),
    DeleteSelection(DeleteSelection),
    Remove(RemoveChar),
    SelectBetween(SelectBetween),
    Undo(Undo),
    Redo(Redo),
    Composed(Composed),
}

#[enum_dispatch]
pub trait Execute {
    fn execute(&mut self, state: &mut EditorState);
}

#[derive(Clone, Debug)]
pub struct SwitchMode(pub EditorMode);

impl Execute for SwitchMode {
    fn execute(&mut self, state: &mut EditorState) {
        match self.0 {
            EditorMode::Normal => {
                state.selection = None;
                state.cursor.col = state.cursor.col.min(len_col(&state).saturating_sub(1));
            }
            EditorMode::Visual => {
                state.selection = Some(Selection::new(state.cursor, state.cursor));
            }
            EditorMode::Insert => {}
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
        state.cursor.col += 1;
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
    use crate::Lines;

    use super::*;
    fn test_state() -> EditorState {
        EditorState::new(Lines::from("Hello World!\n\n123."))
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
}

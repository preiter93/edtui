pub mod delete;
pub mod insert;
pub mod motion;
pub mod select;
use crate::{EditorMode, EditorState};
use enum_dispatch::enum_dispatch;

pub use self::delete::{Delete, DeleteChar, DeleteLine, DeleteSelection, Remove};
pub use self::insert::{Insert, InsertChar, InsertNewline};
pub use self::motion::{Move, MoveBackward, MoveDown, MoveForward, MoveUp};
pub use self::select::{Select, SelectBetween};

#[enum_dispatch(Execute)]
#[derive(Clone, Debug)]
pub enum Action {
    SwitchMode(SwitchMode),
    Append(Append),
    Motion(Move),
    Insert(Insert),
    Delete(Delete),
    Select(Select),
    Undo(Undo),
    Redo(Redo),
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
                state.clear_selection();
                state.cursor.column = state.cursor.column.min(state.len_col().saturating_sub(1));
            }
            EditorMode::Visual => state.set_selection(state.cursor, state.cursor),
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
        state.cursor.column += 1;
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

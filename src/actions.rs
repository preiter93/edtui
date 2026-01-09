//! Editor actions such as move, insert, delete
pub mod cpaste;
pub mod delete;
pub mod insert;
pub mod motion;
pub mod search;
pub mod select;
#[cfg(feature = "system-editor")]
pub mod system_editor;
use crate::state::selection::Selection;
use crate::{EditorMode, EditorState};
use cpaste::PasteOverSelection;
use delete::DeleteToEndOfLine;
use enum_dispatch::enum_dispatch;
use motion::{MoveToFirstRow, MoveToLastRow};
#[cfg(feature = "system-editor")]
pub use system_editor::OpenSystemEditor;

pub use self::cpaste::{CopyLine, CopySelection, Paste};
pub use self::delete::{
    DeleteChar, DeleteCharForward, DeleteLine, DeleteSelection, DeleteToFirstCharOfLine,
    JoinLineWithLineBelow, RemoveChar, ReplaceChar,
};
pub use self::insert::{AppendNewline, InsertChar, InsertNewline, LineBreak};
pub use self::motion::{
    MoveBackward, MoveDown, MoveForward, MoveHalfPageDown, MoveHalfPageUp, MoveToEndOfLine,
    MoveToFirst, MoveToMatchinBracket, MoveToStartOfLine, MoveUp, MoveWordBackward,
    MoveWordForward, MoveWordForwardToEndOfWord,
};
use self::search::StartSearch;
pub use self::search::{
    AppendCharToSearch, FindFirst, FindNext, FindPrevious, RemoveCharFromSearch,
    SelectCurrentSearch, StopSearch,
};
pub use self::select::{
    ChangeInnerBetween, ChangeInnerWord, ChangeSelection, SelectInnerBetween, SelectInnerWord,
    SelectLine,
};

#[enum_dispatch(Execute)]
#[derive(Clone, Debug)]
pub enum Action {
    SwitchMode(SwitchMode),
    MoveForward(MoveForward),
    MoveBackward(MoveBackward),
    MoveUp(MoveUp),
    MoveDown(MoveDown),
    MoveWordForward(MoveWordForward),
    MoveWordForwardToEndOfWord(MoveWordForwardToEndOfWord),
    MoveWordBackward(MoveWordBackward),
    MoveToStartOfLine(MoveToStartOfLine),
    MoveToFirst(MoveToFirst),
    MoveToEndOfLine(MoveToEndOfLine),
    MoveToFirstRow(MoveToFirstRow),
    MoveToLastRow(MoveToLastRow),
    MoveToMatchingBracket(MoveToMatchinBracket),
    MoveHalfPageDown(MoveHalfPageDown),
    MoveHalfPageUp(MoveHalfPageUp),
    InsertChar(InsertChar),
    LineBreak(LineBreak),
    AppendNewline(AppendNewline),
    InsertNewline(InsertNewline),
    ReplaceChar(ReplaceChar),
    RemoveChar(RemoveChar),
    DeleteChar(DeleteChar),
    DeleteCharForward(DeleteCharForward),
    DeleteLine(DeleteLine),
    DeleteToFirstCharOfLine(DeleteToFirstCharOfLine),
    DeleteToEndOfLine(DeleteToEndOfLine),
    DeleteSelection(DeleteSelection),
    JoinLineWithLineBelow(JoinLineWithLineBelow),
    SelectInnerBetween(SelectInnerBetween),
    SelectInnerWord(SelectInnerWord),
    ChangeInnerBetween(ChangeInnerBetween),
    ChangeInnerWord(ChangeInnerWord),
    ChangeSelection(ChangeSelection),
    SelectLine(SelectLine),
    Undo(Undo),
    Redo(Redo),
    Paste(Paste),
    PasteOverSelection(PasteOverSelection),
    CopySelection(CopySelection),
    CopyLine(CopyLine),
    Composed(Composed),
    StartSearch(StartSearch),
    StopSearch(StopSearch),
    FindFirst(FindFirst),
    FindNext(FindNext),
    FindPrevious(FindPrevious),
    SelectCurrentSearch(SelectCurrentSearch),
    AppendCharToSearch(AppendCharToSearch),
    RemoveCharFromSearch(RemoveCharFromSearch),
    #[cfg(feature = "system-editor")]
    OpenSystemEditor(OpenSystemEditor),
}

#[enum_dispatch]
pub trait Execute {
    fn execute(&mut self, state: &mut EditorState);
}

#[derive(Clone, Debug)]
pub struct SwitchMode(pub EditorMode);

impl Execute for SwitchMode {
    fn execute(&mut self, state: &mut EditorState) {
        state.clamp_column();
        match self.0 {
            EditorMode::Normal => {
                state.selection = None;
            }
            EditorMode::Visual => {
                state.selection = Some(Selection::new(state.cursor, state.cursor));
            }
            EditorMode::Insert => {
                if ![EditorMode::Insert, EditorMode::Search].contains(&state.mode) {
                    state.capture();
                }
            }
            EditorMode::Search => {}
        }
        state.mode = self.0;
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
}

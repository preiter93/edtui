//! Editor actions such as move, insert, delete
pub mod change;
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

pub use self::change::{
    ChangeBigWord, ChangeFindForward, ChangeInnerBetween, ChangeInnerBigWord, ChangeInnerWord,
    ChangeSelection, ChangeTillForward, ChangeWord,
};
pub use self::cpaste::{CopyLine, CopySelection, Paste, PasteBefore};
pub use self::delete::{
    DeleteBigWordEnd, DeleteBigWordForward, DeleteChar, DeleteCharForward, DeleteFindForward,
    DeleteLine, DeleteSelection, DeleteTillForward, DeleteToFirstCharOfLine, DeleteWordBackward,
    DeleteWordEnd, DeleteWordForward, JoinLineWithLineBelow, RemoveChar, ReplaceChar,
};
pub use self::insert::{AppendNewline, InsertChar, InsertNewline, LineBreak};
pub use self::motion::{
    FindForward, MoveBackward, MoveDown, MoveForward, MoveHalfPageDown, MoveHalfPageUp,
    MovePageDown, MovePageUp, MoveParagraphBackward, MoveParagraphForward, MoveToEndOfLine,
    MoveToFirst, MoveToMatchinBracket, MoveToStartOfLine, MoveUp, MoveWordBackward,
    MoveWordForward, MoveWordForwardToEndOfWord, TillForward,
};
use self::search::StartSearch;
pub use self::search::{
    AppendCharToSearch, FindFirst, FindNext, FindPrevious, RemoveCharFromSearch,
    SelectCurrentSearch, StopSearch,
};
pub use self::select::{
    DeleteInnerBetween, DeleteInnerBigWord, DeleteInnerWord, SelectInnerBetween,
    SelectInnerBigWord, SelectInnerWord, SelectLine,
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
    MovePageDown(MovePageDown),
    MovePageUp(MovePageUp),
    MoveParagraphForward(MoveParagraphForward),
    MoveParagraphBackward(MoveParagraphBackward),
    FindForward(FindForward),
    TillForward(TillForward),
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
    DeleteWordForward(DeleteWordForward),
    DeleteBigWordForward(DeleteBigWordForward),
    DeleteFindForward(DeleteFindForward),
    DeleteTillForward(DeleteTillForward),
    DeleteWordEnd(DeleteWordEnd),
    DeleteBigWordEnd(DeleteBigWordEnd),
    ChangeWord(ChangeWord),
    ChangeBigWord(ChangeBigWord),
    ChangeFindForward(ChangeFindForward),
    ChangeTillForward(ChangeTillForward),
    DeleteWordBackward(DeleteWordBackward),
    DeleteSelection(DeleteSelection),
    JoinLineWithLineBelow(JoinLineWithLineBelow),
    SelectInnerBetween(SelectInnerBetween),
    SelectInnerWord(SelectInnerWord),
    ChangeInnerBetween(ChangeInnerBetween),
    DeleteInnerBetween(DeleteInnerBetween),
    ChangeInnerWord(ChangeInnerWord),
    DeleteInnerWord(DeleteInnerWord),
    SelectInnerBigWord(SelectInnerBigWord),
    ChangeInnerBigWord(ChangeInnerBigWord),
    DeleteInnerBigWord(DeleteInnerBigWord),
    ChangeSelection(ChangeSelection),
    SelectLine(SelectLine),
    Undo(Undo),
    Redo(Redo),
    RepeatLastChange(RepeatLastChange),
    Paste(Paste),
    PasteBefore(PasteBefore),
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

    /// Whether this action can be replayed by the dot-repeat command.
    fn is_repeatable(&self) -> bool {
        false
    }

    /// Returns a handle to this action's character argument if it takes one
    /// (like `f`/`t`). While the inner value is `None`, the action is still
    /// waiting for the key handler to supply the next keystroke through it.
    fn char_arg(&mut self) -> Option<&mut Option<char>> {
        None
    }
}

pub trait Chainable {
    fn chain<A: Into<Action>>(self, action: A) -> Composed;
}

impl<T: Into<Action>> Chainable for T {
    fn chain<A: Into<Action>>(self, action: A) -> Composed {
        Composed::new(self.into()).chain(action)
    }
}

#[derive(Clone, Debug)]
pub struct SwitchMode(pub EditorMode);

impl Execute for SwitchMode {
    fn execute(&mut self, state: &mut EditorState) {
        let from_insert = state.mode == EditorMode::Insert;

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

        if self.0 == EditorMode::Normal {
            // When leaving insert mode, move the cursor one column left so it
            // rests on the last typed character rather than the empty slot
            // after it.
            if from_insert && state.cursor.col > 0 {
                state.cursor.col -= 1;
            }

            // Re-clamp so the cursor never lingers past the end of the line.
            state.clamp_column();
        }
    }

    fn is_repeatable(&self) -> bool {
        // Entering insert mode begins a repeatable insert session (`.`).
        self.0 == EditorMode::Insert
    }
}

#[derive(Clone, Debug)]
pub struct Undo;

impl Execute for Undo {
    fn execute(&mut self, state: &mut EditorState) {
        state.undo();
    }
}

/// Repeats the last buffer-changing command (dot-repeat).
#[derive(Clone, Debug)]
pub struct RepeatLastChange;

impl Execute for RepeatLastChange {
    fn execute(&mut self, state: &mut EditorState) {
        let Some(mut action) = state.last_change.clone() else {
            return;
        };
        action.execute(state);

        if let Some(text) = state.last_insert.clone() {
            for c in text.chars() {
                InsertChar(c).execute(state);
            }
            SwitchMode(EditorMode::Normal).execute(state);
        }
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

    fn is_repeatable(&self) -> bool {
        self.0.iter().any(Execute::is_repeatable)
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

    #[test]
    fn test_chainable_actions() {
        let mut state = test_state();
        assert_eq!(state.mode, EditorMode::Normal);

        // Test the new chainable syntax: SwitchMode().chain().chain()
        let mut action = SwitchMode(EditorMode::Insert)
            .chain(MoveToEndOfLine())
            .chain(SwitchMode(EditorMode::Visual));

        action.execute(&mut state);

        // Verify the final state after chaining
        assert_eq!(state.mode, EditorMode::Visual);
        assert!(state.selection.is_some());
    }
}

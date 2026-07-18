//! Change actions.
//!
//! A "change" is a delete that leaves the editor in insert mode. Each change
//! command delegates to the corresponding delete primitive and then switches to
//! insert mode, mirroring Vim's `c` operator.

use super::delete::{delete_selection, DeleteBigWordEnd, DeleteWordEnd};
use super::select::{DeleteInnerBetween, DeleteInnerBigWord, DeleteInnerWord};
use super::Execute;
use crate::clipboard::ClipboardTrait;
use crate::{EditorMode, EditorState};

/// Changes from the cursor to the end of the current word: deletes it and
/// enters insert mode (Vim `cw`).
#[derive(Clone, Debug, Copy)]
pub struct ChangeWord(pub usize);

impl Execute for ChangeWord {
    fn execute(&mut self, state: &mut EditorState) {
        DeleteWordEnd(self.0).execute(state);
        state.mode = EditorMode::Insert;
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

/// Changes from the cursor to the end of the current WORD: deletes it and
/// enters insert mode (Vim `cW`).
#[derive(Clone, Debug, Copy)]
pub struct ChangeBigWord(pub usize);

impl Execute for ChangeBigWord {
    fn execute(&mut self, state: &mut EditorState) {
        DeleteBigWordEnd(self.0).execute(state);
        state.mode = EditorMode::Insert;
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

/// Changes the inner word under the cursor: deletes it and enters insert mode.
/// This is the `ciw` primitive.
#[derive(Clone, Debug, Copy)]
pub struct ChangeInnerWord;

impl Execute for ChangeInnerWord {
    fn execute(&mut self, state: &mut EditorState) {
        DeleteInnerWord.execute(state);
        state.mode = EditorMode::Insert;
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

/// Changes the inner WORD under the cursor: deletes it and enters insert mode.
/// This is the `ciW` primitive.
#[derive(Clone, Debug, Copy)]
pub struct ChangeInnerBigWord;

impl Execute for ChangeInnerBigWord {
    fn execute(&mut self, state: &mut EditorState) {
        DeleteInnerBigWord.execute(state);
        state.mode = EditorMode::Insert;
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

/// Changes the text between the given delimiters: deletes the inner content and
/// enters insert mode. This is the `ci<delim>` primitive.
#[derive(Clone, Debug, Copy)]
pub struct ChangeInnerBetween {
    opening: char,
    closing: char,
}

impl ChangeInnerBetween {
    #[must_use]
    pub fn new(opening: char, closing: char) -> Self {
        Self { opening, closing }
    }
}

impl Execute for ChangeInnerBetween {
    fn execute(&mut self, state: &mut EditorState) {
        DeleteInnerBetween::new(self.opening, self.closing).execute(state);
        state.mode = EditorMode::Insert;
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

/// Changes the current selection: deletes it and enters insert mode.
#[derive(Clone, Debug, Copy)]
pub struct ChangeSelection;

impl Execute for ChangeSelection {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(selection) = state.selection.take() {
            state.capture();
            let deleted = delete_selection(state, &selection);
            state.clip.set_text(deleted.into());
        }
    }

    fn is_repeatable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Index2, Lines};

    #[test]
    fn test_change_word_enters_insert_mode() {
        let mut state = EditorState::new(Lines::from("Hello World"));
        state.cursor = Index2::new(0, 0);

        ChangeWord(1).execute(&mut state);
        assert_eq!(state.lines.to_string(), " World");
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.mode, EditorMode::Insert);
    }

    #[test]
    fn test_change_big_word_enters_insert_mode() {
        let mut state = EditorState::new(Lines::from("foo.bar baz"));
        state.cursor = Index2::new(0, 0);

        ChangeBigWord(1).execute(&mut state);
        // `cW` changes the whole WORD but keeps the trailing whitespace.
        assert_eq!(state.lines.to_string(), " baz");
        assert_eq!(state.mode, EditorMode::Insert);
    }

    #[test]
    fn test_change_inner_word() {
        let mut state = EditorState::new(Lines::from("Hello World"));
        state.cursor = Index2::new(0, 1);

        ChangeInnerWord.execute(&mut state);

        assert_eq!(state.lines.to_string(), " World");
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.mode, EditorMode::Insert);
    }

    #[test]
    fn test_change_inner_word_with_punctuation() {
        let mut state = EditorState::new(Lines::from("foo.bar baz"));
        state.cursor = Index2::new(0, 0);

        ChangeInnerWord.execute(&mut state);

        assert_eq!(state.lines.to_string(), ".bar baz");
        assert_eq!(state.cursor, Index2::new(0, 0));
    }

    #[test]
    fn test_change_inner_big_word() {
        let mut state = EditorState::new(Lines::from("foo.bar baz"));
        state.cursor = Index2::new(0, 1);

        ChangeInnerBigWord.execute(&mut state);

        assert_eq!(state.lines.to_string(), " baz");
        assert_eq!(state.cursor, Index2::new(0, 0));
    }

    #[test]
    fn test_change_inner_big_word_on_punctuation() {
        let mut state = EditorState::new(Lines::from("foo.bar baz"));
        state.cursor = Index2::new(0, 3);

        ChangeInnerBigWord.execute(&mut state);

        assert_eq!(state.lines.to_string(), " baz");
        assert_eq!(state.cursor, Index2::new(0, 0));
    }

    #[test]
    fn test_change_inner_between_enters_insert_mode() {
        let mut state = EditorState::new(Lines::from("\"Hello\" World"));
        state.cursor = Index2::new(0, 1);

        ChangeInnerBetween::new('"', '"').execute(&mut state);

        assert_eq!(state.lines.to_string(), "\"\" World");
        assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.mode, EditorMode::Insert);
    }
}

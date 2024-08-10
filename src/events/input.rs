//! Handles key input events
use super::key::KeyEvent;
use crate::actions::search::StartSearch;
use crate::actions::{
    Action, Append, AppendCharToSearch, AppendNewline, Composed, CopySelection, DeleteChar,
    DeleteLine, DeleteSelection, Execute, FindNext, FindPrevious, InsertChar, InsertNewline,
    LineBreak, MoveBackward, MoveDown, MoveForward, MoveToEnd, MoveToFirst, MoveToStart, MoveUp,
    MoveWordBackward, MoveWordForward, Paste, Redo, RemoveChar, RemoveCharFromSearch,
    SelectBetween, SelectLine, StopSearch, SwitchMode, TriggerSearch, Undo,
};
use crate::{EditorMode, EditorState};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct EditorInput {
    lookup: Vec<KeyEvent>,
    register: HashMap<EditorInputKey, Action>,
}

impl Default for EditorInput {
    fn default() -> Self {
        let register: HashMap<EditorInputKey, Action> = HashMap::from([
            // Go into normal mode
            (
                EditorInputKey::i(vec![KeyEvent::Esc]),
                SwitchMode(EditorMode::Normal).into(),
            ),
            (
                EditorInputKey::v(vec![KeyEvent::Esc]),
                SwitchMode(EditorMode::Normal).into(),
            ),
            // Go into insert mode
            (
                EditorInputKey::n(vec![KeyEvent::Char('i')]),
                SwitchMode(EditorMode::Insert).into(),
            ),
            // Go into visual mode
            (
                EditorInputKey::n(vec![KeyEvent::Char('v')]),
                SwitchMode(EditorMode::Visual).into(),
            ),
            // Goes into search mode and starts of a new search.
            (
                EditorInputKey::n(vec![KeyEvent::Char('/')]),
                StartSearch.into(),
            ),
            // Trigger initial search
            (
                EditorInputKey::s(vec![KeyEvent::Enter]),
                TriggerSearch.into(),
            ),
            // Find next
            (
                EditorInputKey::n(vec![KeyEvent::Char('n')]),
                FindNext.into(),
            ),
            // Find previous
            (
                EditorInputKey::n(vec![KeyEvent::Char('N')]),
                FindPrevious.into(),
            ),
            // Clear search
            (EditorInputKey::s(vec![KeyEvent::Esc]), StopSearch.into()),
            // Delete last character from search
            (
                EditorInputKey::s(vec![KeyEvent::Backspace]),
                RemoveCharFromSearch.into(),
            ),
            // Go into insert mode and move one char forward
            (EditorInputKey::n(vec![KeyEvent::Char('a')]), Append.into()),
            // Move cursor forward
            (
                EditorInputKey::n(vec![KeyEvent::Char('l')]),
                MoveForward(1).into(),
            ),
            (
                EditorInputKey::v(vec![KeyEvent::Char('l')]),
                MoveForward(1).into(),
            ),
            (
                EditorInputKey::n(vec![KeyEvent::Right]),
                MoveForward(1).into(),
            ),
            (
                EditorInputKey::v(vec![KeyEvent::Right]),
                MoveForward(1).into(),
            ),
            (
                EditorInputKey::i(vec![KeyEvent::Right]),
                MoveForward(1).into(),
            ),
            // Move cursor backward
            (
                EditorInputKey::n(vec![KeyEvent::Char('h')]),
                MoveBackward(1).into(),
            ),
            (
                EditorInputKey::v(vec![KeyEvent::Char('h')]),
                MoveBackward(1).into(),
            ),
            (
                EditorInputKey::n(vec![KeyEvent::Left]),
                MoveBackward(1).into(),
            ),
            (
                EditorInputKey::v(vec![KeyEvent::Left]),
                MoveBackward(1).into(),
            ),
            (
                EditorInputKey::i(vec![KeyEvent::Left]),
                MoveBackward(1).into(),
            ),
            // Move cursor up
            (
                EditorInputKey::n(vec![KeyEvent::Char('k')]),
                MoveUp(1).into(),
            ),
            (
                EditorInputKey::v(vec![KeyEvent::Char('k')]),
                MoveUp(1).into(),
            ),
            (EditorInputKey::n(vec![KeyEvent::Up]), MoveUp(1).into()),
            (EditorInputKey::v(vec![KeyEvent::Up]), MoveUp(1).into()),
            (EditorInputKey::i(vec![KeyEvent::Up]), MoveUp(1).into()),
            // Move cursor down
            (
                EditorInputKey::n(vec![KeyEvent::Char('j')]),
                MoveDown(1).into(),
            ),
            (
                EditorInputKey::v(vec![KeyEvent::Char('j')]),
                MoveDown(1).into(),
            ),
            (EditorInputKey::n(vec![KeyEvent::Down]), MoveDown(1).into()),
            (EditorInputKey::v(vec![KeyEvent::Down]), MoveDown(1).into()),
            (EditorInputKey::i(vec![KeyEvent::Down]), MoveDown(1).into()),
            // Move one word forward/backward
            (
                EditorInputKey::n(vec![KeyEvent::Char('w')]),
                MoveWordForward(1).into(),
            ),
            (
                EditorInputKey::n(vec![KeyEvent::Char('b')]),
                MoveWordBackward(1).into(),
            ),
            (
                EditorInputKey::v(vec![KeyEvent::Char('w')]),
                MoveWordForward(1).into(),
            ),
            (
                EditorInputKey::v(vec![KeyEvent::Char('b')]),
                MoveWordBackward(1).into(),
            ),
            // Move cursor to start/first/last position
            (
                EditorInputKey::n(vec![KeyEvent::Char('0')]),
                MoveToStart().into(),
            ),
            (
                EditorInputKey::n(vec![KeyEvent::Char('_')]),
                MoveToFirst().into(),
            ),
            (
                EditorInputKey::n(vec![KeyEvent::Char('$')]),
                MoveToEnd().into(),
            ),
            (
                EditorInputKey::v(vec![KeyEvent::Char('0')]),
                MoveToStart().into(),
            ),
            (
                EditorInputKey::v(vec![KeyEvent::Char('_')]),
                MoveToFirst().into(),
            ),
            (
                EditorInputKey::v(vec![KeyEvent::Char('$')]),
                MoveToEnd().into(),
            ),
            // Move cursor to start/first/last position and enter insert mode
            (
                EditorInputKey::n(vec![KeyEvent::Char('I')]),
                Composed::new(MoveToFirst())
                    .chain(SwitchMode(EditorMode::Insert))
                    .into(),
            ),
            (
                EditorInputKey::n(vec![KeyEvent::Char('A')]),
                Composed::new(MoveToEnd()).chain(Append).into(),
            ),
            // Append/insert new line and switch into insert mode
            (
                EditorInputKey::n(vec![KeyEvent::Char('o')]),
                Composed::new(AppendNewline(1))
                    .chain(SwitchMode(EditorMode::Insert))
                    .into(),
            ),
            (
                EditorInputKey::n(vec![KeyEvent::Char('O')]),
                Composed::new(InsertNewline(1))
                    .chain(SwitchMode(EditorMode::Insert))
                    .into(),
            ),
            // Insert a line break
            (
                EditorInputKey::i(vec![KeyEvent::Enter]),
                LineBreak(1).into(),
            ),
            // Remove the current character
            (
                EditorInputKey::n(vec![KeyEvent::Char('x')]),
                RemoveChar(1).into(),
            ),
            // Delete the previous character
            (
                EditorInputKey::i(vec![KeyEvent::Backspace]),
                DeleteChar(1).into(),
            ),
            // Delete the current line
            (
                EditorInputKey::n(vec![KeyEvent::Char('d'), KeyEvent::Char('d')]),
                DeleteLine(1).into(),
            ),
            // Delete the current selection
            (
                EditorInputKey::v(vec![KeyEvent::Char('d')]),
                DeleteSelection.into(),
            ),
            // Select inner word between delimiters
            (
                EditorInputKey::n(vec![
                    KeyEvent::Char('c'),
                    KeyEvent::Char('i'),
                    KeyEvent::Char('w'),
                ]),
                SelectBetween('"').into(),
            ),
            // Select  the line
            (
                EditorInputKey::n(vec![KeyEvent::Char('V')]),
                SelectLine.into(),
            ),
            // Undo
            (EditorInputKey::n(vec![KeyEvent::Char('u')]), Undo.into()),
            // Redo
            (EditorInputKey::n(vec![KeyEvent::Ctrl('r')]), Redo.into()),
            // Copy
            (
                EditorInputKey::v(vec![KeyEvent::Char('y')]),
                CopySelection.into(),
            ),
            // Paste
            (EditorInputKey::n(vec![KeyEvent::Char('p')]), Paste.into()),
            (EditorInputKey::v(vec![KeyEvent::Char('p')]), Paste.into()),
        ]);

        Self {
            lookup: Vec::new(),
            register,
        }
    }
}
impl EditorInput {
    /// Creates a new EditorInput.
    #[must_use]
    pub fn new(register: HashMap<EditorInputKey, Action>) -> Self {
        Self {
            lookup: Vec::new(),
            register,
        }
    }

    /// Insert a new callback to the registry
    pub fn insert<T>(&mut self, key: EditorInputKey, action: T)
    where
        T: Into<Action>,
    {
        self.register.insert(key, action.into());
    }

    /// Extents the register with the contents of an iterator
    pub fn extend<T, U>(&mut self, iter: T)
    where
        U: Into<Action>,
        T: IntoIterator<Item = (EditorInputKey, U)>,
    {
        self.register
            .extend(iter.into_iter().map(|(k, v)| (k, v.into())));
    }

    /// Returns an action for a specific register key, if present.
    /// Returns an action only if there is an exact match. If there
    /// are multiple matches or an inexact match, the specified key
    /// is appended to the lookup vector.
    /// If there is an exact match or if none of the keys in the registry
    /// starts with the current sequence, the lookup sequence is reset.
    #[must_use]
    fn get(&mut self, c: KeyEvent, mode: EditorMode) -> Option<Action> {
        self.lookup.push(c);
        let key = EditorInputKey::new(self.lookup.clone(), mode);

        match self
            .register
            .keys()
            .filter(|k| k.mode == key.mode && k.keys.starts_with(&key.keys))
            .count()
        {
            0 => {
                self.lookup.clear();
                None
            }
            1 => self.register.get(&key).map(|action| {
                self.lookup.clear();
                action.clone()
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct EditorInputKey {
    keys: Vec<KeyEvent>,
    mode: EditorMode,
}

type RegisterCB = fn(&mut EditorState);

#[derive(Clone, Debug)]
struct RegisterVal(pub fn(&mut EditorState));

impl EditorInputKey {
    pub fn new<T>(key: T, mode: EditorMode) -> Self
    where
        T: Into<Vec<KeyEvent>>,
    {
        Self {
            keys: key.into(),
            mode,
        }
    }

    pub fn n<T>(key: T) -> Self
    where
        T: Into<Vec<KeyEvent>>,
    {
        Self::new(key.into(), EditorMode::Normal)
    }

    pub fn v<T>(key: T) -> Self
    where
        T: Into<Vec<KeyEvent>>,
    {
        Self::new(key.into(), EditorMode::Visual)
    }

    pub fn i<T>(key: T) -> Self
    where
        T: Into<Vec<KeyEvent>>,
    {
        Self::new(key.into(), EditorMode::Insert)
    }

    pub fn s<T>(key: T) -> Self
    where
        T: Into<Vec<KeyEvent>>,
    {
        Self::new(key.into(), EditorMode::Search)
    }
}

impl EditorInput {
    pub fn on_key<T>(&mut self, key: T, state: &mut EditorState)
    where
        T: Into<KeyEvent> + Copy,
    {
        let mode = state.mode;

        match key.into() {
            // Always insert characters in insert mode
            KeyEvent::Char(c) if mode == EditorMode::Insert => InsertChar(c).execute(state),
            // Always add characters to search in search mode
            KeyEvent::Char(c) if mode == EditorMode::Search => AppendCharToSearch(c).execute(state),
            // Else lookup an action from the register
            _ => {
                if let Some(mut action) = self.get(key.into(), mode) {
                    action.execute(state);
                }
            }
        }
    }
}

use std::collections::HashMap;

use super::key::KeyEvent;
use crate::actions::search::StartSearch;
use crate::actions::{
    Action, Append, AppendNewline, Composed, CopySelection, DeleteChar, DeleteLine,
    DeleteSelection, FindNext, FindPrevious, InsertNewline, LineBreak, MoveBackward, MoveDown,
    MoveForward, MoveToEnd, MoveToFirst, MoveToStart, MoveUp, MoveWordBackward, MoveWordForward,
    Paste, Redo, RemoveChar, RemoveCharFromSearch, SelectBetween, SelectLine, StopSearch,
    SwitchMode, TriggerSearch, Undo,
};
use crate::{EditorMode, EditorState};

#[derive(Clone, Debug)]
pub struct Register {
    lookup: Vec<KeyEvent>,
    register: HashMap<RegisterKey, Action>,
}

impl Default for Register {
    fn default() -> Self {
        let register: HashMap<RegisterKey, Action> = HashMap::from([
            // Go into normal mode
            (
                RegisterKey::i(vec![KeyEvent::Esc]),
                SwitchMode(EditorMode::Normal).into(),
            ),
            (
                RegisterKey::v(vec![KeyEvent::Esc]),
                SwitchMode(EditorMode::Normal).into(),
            ),
            // Go into insert mode
            (
                RegisterKey::n(vec![KeyEvent::Char('i')]),
                SwitchMode(EditorMode::Insert).into(),
            ),
            // Go into visual mode
            (
                RegisterKey::n(vec![KeyEvent::Char('v')]),
                SwitchMode(EditorMode::Visual).into(),
            ),
            // Goes into search mode and starts of a new search.
            (
                RegisterKey::n(vec![KeyEvent::Char('/')]),
                StartSearch.into(),
            ),
            // Trigger initial search
            (RegisterKey::s(vec![KeyEvent::Enter]), TriggerSearch.into()),
            // Find next
            (RegisterKey::n(vec![KeyEvent::Char('n')]), FindNext.into()),
            // Find previous
            (
                RegisterKey::n(vec![KeyEvent::Char('N')]),
                FindPrevious.into(),
            ),
            // Clear search
            (RegisterKey::s(vec![KeyEvent::Esc]), StopSearch.into()),
            // Delete last character from search
            (
                RegisterKey::s(vec![KeyEvent::Backspace]),
                RemoveCharFromSearch.into(),
            ),
            // Go into insert mode and move one char forward
            (RegisterKey::n(vec![KeyEvent::Char('a')]), Append.into()),
            // Move cursor forward
            (
                RegisterKey::n(vec![KeyEvent::Char('l')]),
                MoveForward(1).into(),
            ),
            (
                RegisterKey::v(vec![KeyEvent::Char('l')]),
                MoveForward(1).into(),
            ),
            (RegisterKey::n(vec![KeyEvent::Right]), MoveForward(1).into()),
            (RegisterKey::v(vec![KeyEvent::Right]), MoveForward(1).into()),
            (RegisterKey::i(vec![KeyEvent::Right]), MoveForward(1).into()),
            // Move cursor backward
            (
                RegisterKey::n(vec![KeyEvent::Char('h')]),
                MoveBackward(1).into(),
            ),
            (
                RegisterKey::v(vec![KeyEvent::Char('h')]),
                MoveBackward(1).into(),
            ),
            (RegisterKey::n(vec![KeyEvent::Left]), MoveBackward(1).into()),
            (RegisterKey::v(vec![KeyEvent::Left]), MoveBackward(1).into()),
            (RegisterKey::i(vec![KeyEvent::Left]), MoveBackward(1).into()),
            // Move cursor up
            (RegisterKey::n(vec![KeyEvent::Char('k')]), MoveUp(1).into()),
            (RegisterKey::v(vec![KeyEvent::Char('k')]), MoveUp(1).into()),
            (RegisterKey::n(vec![KeyEvent::Up]), MoveUp(1).into()),
            (RegisterKey::v(vec![KeyEvent::Up]), MoveUp(1).into()),
            (RegisterKey::i(vec![KeyEvent::Up]), MoveUp(1).into()),
            // Move cursor down
            (
                RegisterKey::n(vec![KeyEvent::Char('j')]),
                MoveDown(1).into(),
            ),
            (
                RegisterKey::v(vec![KeyEvent::Char('j')]),
                MoveDown(1).into(),
            ),
            (RegisterKey::n(vec![KeyEvent::Down]), MoveDown(1).into()),
            (RegisterKey::v(vec![KeyEvent::Down]), MoveDown(1).into()),
            (RegisterKey::i(vec![KeyEvent::Down]), MoveDown(1).into()),
            // Move one word forward/backward
            (
                RegisterKey::n(vec![KeyEvent::Char('w')]),
                MoveWordForward(1).into(),
            ),
            (
                RegisterKey::n(vec![KeyEvent::Char('b')]),
                MoveWordBackward(1).into(),
            ),
            (
                RegisterKey::v(vec![KeyEvent::Char('w')]),
                MoveWordForward(1).into(),
            ),
            (
                RegisterKey::v(vec![KeyEvent::Char('b')]),
                MoveWordBackward(1).into(),
            ),
            // Move cursor to start/first/last position
            (
                RegisterKey::n(vec![KeyEvent::Char('0')]),
                MoveToStart().into(),
            ),
            (
                RegisterKey::n(vec![KeyEvent::Char('_')]),
                MoveToFirst().into(),
            ),
            (
                RegisterKey::n(vec![KeyEvent::Char('$')]),
                MoveToEnd().into(),
            ),
            (
                RegisterKey::v(vec![KeyEvent::Char('0')]),
                MoveToStart().into(),
            ),
            (
                RegisterKey::v(vec![KeyEvent::Char('_')]),
                MoveToFirst().into(),
            ),
            (
                RegisterKey::v(vec![KeyEvent::Char('$')]),
                MoveToEnd().into(),
            ),
            // Move cursor to start/first/last position and enter insert mode
            (
                RegisterKey::n(vec![KeyEvent::Char('I')]),
                Composed::new(MoveToFirst())
                    .chain(SwitchMode(EditorMode::Insert))
                    .into(),
            ),
            (
                RegisterKey::n(vec![KeyEvent::Char('A')]),
                Composed::new(MoveToEnd()).chain(Append).into(),
            ),
            // Append/insert new line and switch into insert mode
            (
                RegisterKey::n(vec![KeyEvent::Char('o')]),
                Composed::new(AppendNewline(1))
                    .chain(SwitchMode(EditorMode::Insert))
                    .into(),
            ),
            (
                RegisterKey::n(vec![KeyEvent::Char('O')]),
                Composed::new(InsertNewline(1))
                    .chain(SwitchMode(EditorMode::Insert))
                    .into(),
            ),
            // Insert a line break
            (RegisterKey::i(vec![KeyEvent::Enter]), LineBreak(1).into()),
            // Remove the current character
            (
                RegisterKey::n(vec![KeyEvent::Char('x')]),
                RemoveChar(1).into(),
            ),
            // Delete the previous character
            (
                RegisterKey::i(vec![KeyEvent::Backspace]),
                DeleteChar(1).into(),
            ),
            // Delete the current line
            (
                RegisterKey::n(vec![KeyEvent::Char('d'), KeyEvent::Char('d')]),
                DeleteLine(1).into(),
            ),
            // Delete the current selection
            (
                RegisterKey::v(vec![KeyEvent::Char('d')]),
                DeleteSelection.into(),
            ),
            // Select inner word between delimiters
            (
                RegisterKey::n(vec![
                    KeyEvent::Char('c'),
                    KeyEvent::Char('i'),
                    KeyEvent::Char('w'),
                ]),
                SelectBetween('"').into(),
            ),
            // Select  the line
            (RegisterKey::n(vec![KeyEvent::Char('V')]), SelectLine.into()),
            // Undo
            (RegisterKey::n(vec![KeyEvent::Char('u')]), Undo.into()),
            // Redo
            (RegisterKey::n(vec![KeyEvent::Ctrl('r')]), Redo.into()),
            // Copy
            (
                RegisterKey::v(vec![KeyEvent::Char('y')]),
                CopySelection.into(),
            ),
            // Paste
            (RegisterKey::n(vec![KeyEvent::Char('p')]), Paste.into()),
            (RegisterKey::v(vec![KeyEvent::Char('p')]), Paste.into()),
        ]);

        Self {
            lookup: Vec::new(),
            register,
        }
    }
}

impl Register {
    /// Constructs a new Register
    #[must_use]
    pub fn new() -> Self {
        Self {
            lookup: Vec::new(),
            register: HashMap::new(),
        }
    }

    /// Insert a new callback to the registry
    pub fn insert<T: Into<Action>>(&mut self, k: RegisterKey, v: T) {
        self.register.insert(k, v.into());
    }

    /// Returns an action for a specific register key, if present.
    /// Returns an action only if there is an exact match. If there
    /// are multiple matches or an inexact match, the specified key
    /// is appended to the lookup vector.
    /// If there is an exact match or if none of the keys in the registry
    /// starts with the current sequence, the lookup sequence is reset.
    #[must_use]
    pub fn get(&mut self, c: KeyEvent, mode: EditorMode) -> Option<Action> {
        let key = self.create_register_key(c, mode);

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

    fn create_register_key(&mut self, c: KeyEvent, mode: EditorMode) -> RegisterKey {
        self.lookup.push(c);
        RegisterKey::new(self.lookup.clone(), mode)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct RegisterKey {
    pub keys: Vec<KeyEvent>,
    pub mode: EditorMode,
}

pub type RegisterCB = fn(&mut EditorState);

#[derive(Clone, Debug)]
pub struct RegisterVal(pub fn(&mut EditorState));

impl RegisterKey {
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

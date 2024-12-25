use crate::actions::cpaste::PasteOverSelection;
use crate::actions::delete::DeleteToEndOfLine;
use crate::actions::motion::{MoveHalfPageDown, MoveToFirstRow, MoveToLastRow};
use crate::actions::search::StartSearch;
use crate::actions::{
    Action, Append, AppendCharToSearch, AppendNewline, ChangeInnerBetween, Composed, CopyLine,
    CopySelection, DeleteChar, DeleteLine, DeleteSelection, Execute, FindNext, FindPrevious,
    InsertChar, InsertNewline, JoinLineWithLineBelow, LineBreak, MoveBackward, MoveDown,
    MoveForward, MoveHalfPageUp, MoveToEndOfLine, MoveToFirst, MoveToMatchinBracket,
    MoveToStartOfLine, MoveUp, MoveWordBackward, MoveWordForward, MoveWordForwardToEndOfWord,
    Paste, Redo, RemoveChar, RemoveCharFromSearch, SelectInnerBetween, SelectLine, StopSearch,
    SwitchMode, TriggerSearch, Undo,
};
use crate::{EditorMode, EditorState};
use ratatui::crossterm::event::{KeyCode, KeyEvent as CTKeyEvent, KeyModifiers};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum KeyEvent {
    Char(char),
    Down,
    Up,
    Right,
    Left,
    Enter,
    Esc,
    Backspace,
    Tab,
    Ctrl(char),
    None,
}

impl From<CTKeyEvent> for KeyEvent {
    fn from(key: CTKeyEvent) -> Self {
        if key.modifiers == KeyModifiers::CONTROL {
            return match key.code {
                KeyCode::Char(c) => KeyEvent::Ctrl(c),
                _ => KeyEvent::None,
            };
        }

        match key.code {
            KeyCode::Char(c) => KeyEvent::Char(c),
            KeyCode::Enter => KeyEvent::Enter,
            KeyCode::Down => KeyEvent::Down,
            KeyCode::Up => KeyEvent::Up,
            KeyCode::Right => KeyEvent::Right,
            KeyCode::Left => KeyEvent::Left,
            KeyCode::Esc => KeyEvent::Esc,
            KeyCode::Backspace => KeyEvent::Backspace,
            KeyCode::Tab => KeyEvent::Tab,
            _ => KeyEvent::None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeyEventHandler {
    lookup: Vec<KeyEvent>,
    register: HashMap<KeyEventRegister, Action>,
}

impl Default for KeyEventHandler {
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        let register: HashMap<KeyEventRegister, Action> = HashMap::from([
            // Go into normal mode
            (
                KeyEventRegister::i(vec![KeyEvent::Esc]),
                SwitchMode(EditorMode::Normal).into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Esc]),
                SwitchMode(EditorMode::Normal).into(),
            ),
            // Go into insert mode
            (
                KeyEventRegister::n(vec![KeyEvent::Char('i')]),
                SwitchMode(EditorMode::Insert).into(),
            ),
            // Go into visual mode
            (
                KeyEventRegister::n(vec![KeyEvent::Char('v')]),
                SwitchMode(EditorMode::Visual).into(),
            ),
            // Goes into search mode and starts of a new search.
            (
                KeyEventRegister::n(vec![KeyEvent::Char('/')]),
                StartSearch.into(),
            ),
            // Trigger initial search
            (
                KeyEventRegister::s(vec![KeyEvent::Enter]),
                TriggerSearch.into(),
            ),
            // Find next
            (
                KeyEventRegister::n(vec![KeyEvent::Char('n')]),
                FindNext.into(),
            ),
            // Find previous
            (
                KeyEventRegister::n(vec![KeyEvent::Char('N')]),
                FindPrevious.into(),
            ),
            // Clear search
            (KeyEventRegister::s(vec![KeyEvent::Esc]), StopSearch.into()),
            // Delete last character from search
            (
                KeyEventRegister::s(vec![KeyEvent::Backspace]),
                RemoveCharFromSearch.into(),
            ),
            // Go into insert mode and move one char forward
            (
                KeyEventRegister::n(vec![KeyEvent::Char('a')]),
                Append.into(),
            ),
            // Move cursor forward
            (
                KeyEventRegister::n(vec![KeyEvent::Char('l')]),
                MoveForward(1).into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('l')]),
                MoveForward(1).into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Right]),
                MoveForward(1).into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Right]),
                MoveForward(1).into(),
            ),
            (
                KeyEventRegister::i(vec![KeyEvent::Right]),
                MoveForward(1).into(),
            ),
            // Move cursor backward
            (
                KeyEventRegister::n(vec![KeyEvent::Char('h')]),
                MoveBackward(1).into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('h')]),
                MoveBackward(1).into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Left]),
                MoveBackward(1).into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Left]),
                MoveBackward(1).into(),
            ),
            (
                KeyEventRegister::i(vec![KeyEvent::Left]),
                MoveBackward(1).into(),
            ),
            // Move cursor up
            (
                KeyEventRegister::n(vec![KeyEvent::Char('k')]),
                MoveUp(1).into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('k')]),
                MoveUp(1).into(),
            ),
            (KeyEventRegister::n(vec![KeyEvent::Up]), MoveUp(1).into()),
            (KeyEventRegister::v(vec![KeyEvent::Up]), MoveUp(1).into()),
            (KeyEventRegister::i(vec![KeyEvent::Up]), MoveUp(1).into()),
            // Move cursor down
            (
                KeyEventRegister::n(vec![KeyEvent::Char('j')]),
                MoveDown(1).into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('j')]),
                MoveDown(1).into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Down]),
                MoveDown(1).into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Down]),
                MoveDown(1).into(),
            ),
            (
                KeyEventRegister::i(vec![KeyEvent::Down]),
                MoveDown(1).into(),
            ),
            // Move one word forward/backward
            (
                KeyEventRegister::n(vec![KeyEvent::Char('w')]),
                MoveWordForward(1).into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('w')]),
                MoveWordForward(1).into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Char('e')]),
                MoveWordForwardToEndOfWord(1).into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('e')]),
                MoveWordForwardToEndOfWord(1).into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Char('b')]),
                MoveWordBackward(1).into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('b')]),
                MoveWordBackward(1).into(),
            ),
            // Move cursor to start/first/last position
            (
                KeyEventRegister::n(vec![KeyEvent::Char('0')]),
                MoveToStartOfLine().into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Char('_')]),
                MoveToFirst().into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Char('$')]),
                MoveToEndOfLine().into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('0')]),
                MoveToStartOfLine().into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('_')]),
                MoveToFirst().into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('$')]),
                MoveToEndOfLine().into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Ctrl('d')]),
                MoveHalfPageDown().into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Ctrl('d')]),
                MoveHalfPageDown().into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Ctrl('u')]),
                MoveHalfPageUp().into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Ctrl('u')]),
                MoveHalfPageUp().into(),
            ),
            // Move cursor to start/first/last position and enter insert mode
            (
                KeyEventRegister::n(vec![KeyEvent::Char('I')]),
                Composed::new(SwitchMode(EditorMode::Insert))
                    .chain(MoveToFirst())
                    .into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Char('A')]),
                Composed::new(Append).chain(MoveToEndOfLine()).into(),
            ),
            // Move cursor to start/last row in the buffer
            (
                KeyEventRegister::n(vec![KeyEvent::Char('g'), KeyEvent::Char('g')]),
                MoveToFirstRow().into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('g'), KeyEvent::Char('g')]),
                MoveToFirstRow().into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Char('G')]),
                MoveToLastRow().into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('G')]),
                MoveToLastRow().into(),
            ),
            // Move cursor to the next opening/closing bracket.
            (
                KeyEventRegister::n(vec![KeyEvent::Char('%')]),
                MoveToMatchinBracket().into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('%')]),
                MoveToMatchinBracket().into(),
            ),
            // Append/insert new line and switch into insert mode
            (
                KeyEventRegister::n(vec![KeyEvent::Char('o')]),
                Composed::new(AppendNewline(1)).into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Char('O')]),
                Composed::new(InsertNewline(1)).into(),
            ),
            // Insert a line break
            (
                KeyEventRegister::i(vec![KeyEvent::Enter]),
                LineBreak(1).into(),
            ),
            // Remove the current character
            (
                KeyEventRegister::n(vec![KeyEvent::Char('x')]),
                RemoveChar(1).into(),
            ),
            // Delete the previous character
            (
                KeyEventRegister::i(vec![KeyEvent::Backspace]),
                DeleteChar(1).into(),
            ),
            // Delete the current line
            (
                KeyEventRegister::n(vec![KeyEvent::Char('d'), KeyEvent::Char('d')]),
                DeleteLine(1).into(),
            ),
            // Delete from the cursor to the end of the line
            (
                KeyEventRegister::n(vec![KeyEvent::Char('D')]),
                DeleteToEndOfLine.into(),
            ),
            // Delete the current selection
            (
                KeyEventRegister::v(vec![KeyEvent::Char('d')]),
                DeleteSelection.into(),
            ),
            // Join the current line with the line below
            (
                KeyEventRegister::n(vec![KeyEvent::Char('J')]),
                JoinLineWithLineBelow.into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('J')]),
                JoinLineWithLineBelow.into(),
            ),
            // Select inner word between delimiters
            (
                KeyEventRegister::v(vec![KeyEvent::Char('i'), KeyEvent::Char('w')]),
                SelectInnerBetween::new('"', '"').into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('i'), KeyEvent::Char('"')]),
                SelectInnerBetween::new('"', '"').into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('i'), KeyEvent::Char('\'')]),
                SelectInnerBetween::new('\'', '\'').into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('i'), KeyEvent::Char('(')]),
                SelectInnerBetween::new('(', ')').into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('i'), KeyEvent::Char(')')]),
                SelectInnerBetween::new('(', ')').into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('i'), KeyEvent::Char('{')]),
                SelectInnerBetween::new('{', '}').into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('i'), KeyEvent::Char('}')]),
                SelectInnerBetween::new('{', '}').into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('i'), KeyEvent::Char('[')]),
                SelectInnerBetween::new('[', ']').into(),
            ),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('i'), KeyEvent::Char(']')]),
                SelectInnerBetween::new('[', ']').into(),
            ),
            // Change inner word between delimiters
            (
                KeyEventRegister::n(vec![
                    KeyEvent::Char('c'),
                    KeyEvent::Char('i'),
                    KeyEvent::Char('w'),
                ]),
                ChangeInnerBetween::new('"', '"').into(),
            ),
            (
                KeyEventRegister::n(vec![
                    KeyEvent::Char('c'),
                    KeyEvent::Char('i'),
                    KeyEvent::Char('"'),
                ]),
                ChangeInnerBetween::new('"', '"').into(),
            ),
            (
                KeyEventRegister::n(vec![
                    KeyEvent::Char('c'),
                    KeyEvent::Char('i'),
                    KeyEvent::Char('\''),
                ]),
                ChangeInnerBetween::new('\'', '\'').into(),
            ),
            (
                KeyEventRegister::n(vec![
                    KeyEvent::Char('c'),
                    KeyEvent::Char('i'),
                    KeyEvent::Char('('),
                ]),
                ChangeInnerBetween::new('(', ')').into(),
            ),
            (
                KeyEventRegister::n(vec![
                    KeyEvent::Char('c'),
                    KeyEvent::Char('i'),
                    KeyEvent::Char(')'),
                ]),
                ChangeInnerBetween::new('(', ')').into(),
            ),
            (
                KeyEventRegister::n(vec![
                    KeyEvent::Char('c'),
                    KeyEvent::Char('i'),
                    KeyEvent::Char('{'),
                ]),
                ChangeInnerBetween::new('{', '}').into(),
            ),
            (
                KeyEventRegister::n(vec![
                    KeyEvent::Char('c'),
                    KeyEvent::Char('i'),
                    KeyEvent::Char('}'),
                ]),
                ChangeInnerBetween::new('{', '}').into(),
            ),
            (
                KeyEventRegister::n(vec![
                    KeyEvent::Char('c'),
                    KeyEvent::Char('i'),
                    KeyEvent::Char('['),
                ]),
                ChangeInnerBetween::new('[', ']').into(),
            ),
            (
                KeyEventRegister::n(vec![
                    KeyEvent::Char('c'),
                    KeyEvent::Char('i'),
                    KeyEvent::Char(']'),
                ]),
                ChangeInnerBetween::new('[', ']').into(),
            ),
            // Select  the line
            (
                KeyEventRegister::n(vec![KeyEvent::Char('V')]),
                SelectLine.into(),
            ),
            // Undo
            (KeyEventRegister::n(vec![KeyEvent::Char('u')]), Undo.into()),
            // Redo
            (KeyEventRegister::n(vec![KeyEvent::Ctrl('r')]), Redo.into()),
            // Copy
            (
                KeyEventRegister::v(vec![KeyEvent::Char('y')]),
                CopySelection.into(),
            ),
            (
                KeyEventRegister::n(vec![KeyEvent::Char('y'), KeyEvent::Char('y')]),
                CopyLine.into(),
            ),
            // Paste
            (KeyEventRegister::n(vec![KeyEvent::Char('p')]), Paste.into()),
            (
                KeyEventRegister::v(vec![KeyEvent::Char('p')]),
                PasteOverSelection.into(),
            ),
        ]);

        Self {
            lookup: Vec::new(),
            register,
        }
    }
}

impl KeyEventHandler {
    /// Creates a new `EditorInput`.
    #[must_use]
    pub fn new(register: HashMap<KeyEventRegister, Action>) -> Self {
        Self {
            lookup: Vec::new(),
            register,
        }
    }

    /// Insert a new callback to the registry
    pub fn insert<T>(&mut self, key: KeyEventRegister, action: T)
    where
        T: Into<Action>,
    {
        self.register.insert(key, action.into());
    }

    /// Extents the register with the contents of an iterator
    pub fn extend<T, U>(&mut self, iter: T)
    where
        U: Into<Action>,
        T: IntoIterator<Item = (KeyEventRegister, U)>,
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
        let key = KeyEventRegister::new(self.lookup.clone(), mode);

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
pub struct KeyEventRegister {
    keys: Vec<KeyEvent>,
    mode: EditorMode,
}

type RegisterCB = fn(&mut EditorState);

#[derive(Clone, Debug)]
struct RegisterVal(pub fn(&mut EditorState));

impl KeyEventRegister {
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

impl KeyEventHandler {
    pub(crate) fn on_event<T>(&mut self, key: T, state: &mut EditorState)
    where
        T: Into<KeyEvent> + Copy,
    {
        let mode = state.mode;

        match key.into() {
            // Always insert characters in insert mode
            KeyEvent::Char(c) if mode == EditorMode::Insert => InsertChar(c).execute(state),
            KeyEvent::Tab if mode == EditorMode::Insert => InsertChar('\t').execute(state),
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
